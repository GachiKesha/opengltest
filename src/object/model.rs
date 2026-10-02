use crate::{
    object::{Mesh, Texture, Vertex},
    shader::Shader,
};
use glad_gl::gl::{
    BindTexture, GLint, GLsizei, GenTextures, GenerateMipmap, LINEAR, LINEAR_MIPMAP_LINEAR, RED,
    REPEAT, RGB, RGBA, TEXTURE_2D, TEXTURE_MAG_FILTER, TEXTURE_MIN_FILTER, TEXTURE_WRAP_S,
    TEXTURE_WRAP_T, TexImage2D, TexParameteri, UNSIGNED_BYTE,
};
use glam::{Vec2, Vec3};
use image::GenericImageView;
use russimp_ng::sys::{
    AI_SCENE_FLAGS_INCOMPLETE, aiGetErrorString, aiGetMaterialTexture, aiGetMaterialTextureCount,
    aiImportFile, aiMaterial, aiMesh, aiNode, aiPostProcessSteps_aiProcess_FlipUVs,
    aiPostProcessSteps_aiProcess_Triangulate, aiReleaseImport, aiScene, aiString, aiTextureType,
    aiTextureType_aiTextureType_DIFFUSE, aiTextureType_aiTextureType_SPECULAR,
};
use std::{
    ffi::{CStr, CString, c_void},
    mem::MaybeUninit,
    path::PathBuf,
    slice::from_raw_parts,
};

pub struct Model {
    meshes: Vec<Mesh>,
    textures_loaded: Vec<Texture>,
    directory: String,
}

impl Model {
    pub fn new(path: &str) -> Self {
        let mut model = Self {
            meshes: Vec::new(),
            textures_loaded: Vec::new(),
            directory: String::new(),
        };
        model.load_model(path);
        model
    }

    fn load_model(&mut self, path: &str) {
        let c_path = CString::new(path).unwrap();
        unsafe {
            let scene = aiImportFile(
                c_path.as_ptr(),
                aiPostProcessSteps_aiProcess_Triangulate | aiPostProcessSteps_aiProcess_FlipUVs,
            );

            if scene.is_null() {
                let error = CStr::from_ptr(aiGetErrorString()).to_string_lossy();
                println!("ERROR::ASSIMP::{}", error);
                return;
            }

            let scene_ref = &*scene;
            if scene_ref.mFlags & AI_SCENE_FLAGS_INCOMPLETE != 0 || scene_ref.mRootNode.is_null() {
                let error = CStr::from_ptr(aiGetErrorString()).to_string_lossy();
                println!("ERROR::ASSIMP::{}", error);
                aiReleaseImport(scene);
                return;
            }

            self.directory = std::path::Path::new(path)
                .parent()
                .unwrap_or(std::path::Path::new(""))
                .to_string_lossy()
                .into_owned();

            self.process_node(&*scene_ref.mRootNode, scene_ref);
            aiReleaseImport(scene);
        }
    }

    fn process_node(&mut self, node: &aiNode, scene: &aiScene) {
        unsafe {
            for i in 0..node.mNumMeshes as usize {
                let mesh = &**scene.mMeshes.add(*node.mMeshes.add(i) as usize);
                let processed_mesh = self.process_mesh(mesh, scene);
                self.meshes.push(processed_mesh);
            }

            for i in 0..node.mNumChildren as usize {
                let child = &**node.mChildren.add(i);
                self.process_node(child, scene);
            }
        }
    }

    fn process_mesh(&mut self, mesh: &aiMesh, scene: &aiScene) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut textures = Vec::new();

        let positions = unsafe { from_raw_parts(mesh.mVertices, mesh.mNumVertices as usize) };

        let normals = if !mesh.mNormals.is_null() {
            Some(unsafe { from_raw_parts(mesh.mNormals, mesh.mNumVertices as usize) })
        } else {
            None
        };

        let tex_coords = if !mesh.mTextureCoords[0].is_null() {
            Some(unsafe { from_raw_parts(mesh.mTextureCoords[0], mesh.mNumVertices as usize) })
        } else {
            None
        };

        for i in 0..positions.len() {
            let position = positions[i];

            let mut vertex = Vertex {
                position: Vec3::new(position.x, position.y, position.z),
                normal: Vec3::ZERO,
                tex_coords: Vec2::ZERO,
            };

            if let Some(normals) = normals {
                let normal = normals[i];
                vertex.normal = Vec3::new(normal.x, normal.y, normal.z);
            }

            if let Some(tex_coords) = tex_coords {
                let tex_coord = tex_coords[i];
                vertex.tex_coords = Vec2::new(tex_coord.x, tex_coord.y);
            }

            vertices.push(vertex);
        }

        let faces = unsafe { from_raw_parts(mesh.mFaces, mesh.mNumFaces as usize) };
        for i in 0..mesh.mNumFaces as usize {
            let face = faces[i];
            let face_indices = unsafe { from_raw_parts(face.mIndices, face.mNumIndices as usize) };
            for face_index in face_indices {
                indices.push(*face_index)
            }
        }

        let material = unsafe { *scene.mMaterials.add(mesh.mMaterialIndex as usize) };
        let diffuse_maps = self.load_material_textures(
            unsafe { &*material },
            &aiTextureType_aiTextureType_DIFFUSE,
            "texture_diffuse",
        );
        textures.extend(diffuse_maps);
        let specular_maps = self.load_material_textures(
            unsafe { &*material },
            &aiTextureType_aiTextureType_SPECULAR,
            "texture_specular",
        );
        textures.extend(specular_maps);

        Mesh::new(vertices, indices, textures)
    }

    fn load_material_textures(
        &mut self,
        mat: &aiMaterial,
        r#type: &aiTextureType,
        type_name: &str,
    ) -> Vec<Texture> {
        let mut textures = Vec::new();

        unsafe {
            for i in 0..aiGetMaterialTextureCount(mat, *r#type) {
                let mut str = MaybeUninit::<aiString>::uninit();

                aiGetMaterialTexture(
                    mat,
                    *r#type,
                    i,
                    str.as_mut_ptr(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                );

                let str = str.assume_init();

                let path = std::ffi::CStr::from_ptr(str.data.as_ptr())
                    .to_string_lossy()
                    .into_owned();

                let mut skip = false;

                for texture in &self.textures_loaded {
                    if texture.path == path {
                        textures.push(texture.clone());
                        skip = true;
                        break;
                    }
                }

                if !skip {
                    let texture = Texture {
                        id: texture_from_file(&path, &self.directory),
                        r#type: type_name.to_string(),
                        path: path.clone(),
                    };

                    textures.push(texture.clone());
                    self.textures_loaded.push(texture);
                }
            }
        }

        textures
    }

    pub fn draw(&self, shader: &Shader) {
        for mesh in &self.meshes {
            mesh.draw(shader);
        }
    }
}

fn texture_from_file(path: &str, directory: &String) -> u32 {
    let filename = PathBuf::from(directory).join(path);
    let mut texture_id = 0;

    unsafe {
        GenTextures(1, &mut texture_id);
    }

    match image::open(filename) {
        Ok(img) => {
            let (width, height) = img.dimensions();
            let (format, data) = match img {
                image::DynamicImage::ImageLuma8(img) => (RED, img.into_raw()),
                image::DynamicImage::ImageRgb8(img) => (RGB, img.into_raw()),
                image::DynamicImage::ImageRgba8(img) => (RGBA, img.into_raw()),
                img => {
                    let img = img.to_rgba8();
                    (RGBA, img.into_raw())
                }
            };

            unsafe {
                BindTexture(TEXTURE_2D, texture_id);
                TexImage2D(
                    TEXTURE_2D,
                    0,
                    format as GLint,
                    width as GLsizei,
                    height as GLsizei,
                    0,
                    format,
                    UNSIGNED_BYTE,
                    data.as_ptr() as *const c_void,
                );
                GenerateMipmap(TEXTURE_2D);

                TexParameteri(TEXTURE_2D, TEXTURE_WRAP_S, REPEAT as GLint);
                TexParameteri(TEXTURE_2D, TEXTURE_WRAP_T, REPEAT as GLint);
                TexParameteri(
                    TEXTURE_2D,
                    TEXTURE_MIN_FILTER,
                    LINEAR_MIPMAP_LINEAR as GLint,
                );
                TexParameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, LINEAR as GLint);
            }
        }
        Err(err) => {
            eprintln!("{}", err);
            println!("Texture failed to load at path: {}", path);
        }
    };

    texture_id
}
