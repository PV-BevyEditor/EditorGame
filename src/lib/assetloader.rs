use bevy::{
    prelude::*,
    render::{
        mesh::PrimitiveTopology::TriangleList,
        render_asset::RenderAssetUsages,
        render_resource::{
            Extent3d,
            TextureDescriptor, 
            TextureDimension, 
            TextureFormat, 
            TextureUsages,
        },
    },
};
use bevy_mesh::Indices;
use rfd::FileDialog;
use std::{
    fs,
    path::PathBuf,
};
use image::load_from_memory;
use gltf;

pub fn loadImage(mut images: ResMut<Assets<Image>>, path: PathBuf) -> Result<Handle<Image>, String> {
    if let Ok(imageData) = fs::read(&path) {
        let decodedImage = load_from_memory(&imageData).expect("Failed to decode image").to_rgba8();
        let dimensions = decodedImage.dimensions();
        let image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: dimensions.0,
                    height: dimensions.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            },
            data: decodedImage.into_raw(),
            ..default()
        };

        return Ok(images.add(image));
    } else {
        return Err("Failed to load selected image.".into());
    }
}

pub fn loadModel(mut meshes: ResMut<Assets<Mesh>>, path: PathBuf) -> Result<Vec<Handle<Mesh>>, String> {
    let (document, buffers, _images) = gltf::import(path).expect("Failed to import gltf file.");

    let mut col: Vec<Handle<Mesh>> = vec![];

    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let positions = primitive.get(&gltf::Semantic::Positions)
                .and_then(|accessor| readBufferData::<[f32; 3]>(&accessor, &buffers)).expect("Failed to get positions from gltf file.");
            
            let normals = primitive.get(&gltf::Semantic::Normals)
                .and_then(|accessor| readBufferData::<[f32; 3]>(&accessor, &buffers));
            
            let uvs = primitive.get(&gltf::Semantic::TexCoords(0))
                .and_then(|accessor| readBufferData::<[f32; 2]>(&accessor, &buffers));
            
            let mut bevyMesh = Mesh::new(TriangleList, RenderAssetUsages::default());
            bevyMesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

            if let Some(normals) = normals {
                bevyMesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            }
            if let Some(uvs) = uvs {
                bevyMesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
            }

            if let Some(indicesAccessor) = primitive.indices() {
                let indices = readIndices(&indicesAccessor, &buffers);
                bevyMesh.insert_indices(Indices::U32(indices));
            }

            col.push(meshes.add(bevyMesh));
        }
    }

    Ok(col)
}

pub fn loadUserImage(images: ResMut<Assets<Image>>) -> Result<Handle<Image>, String> {
    if let Some(path) = FileDialog::new().add_filter("Image", &["png", "jpg", "jpeg"]).pick_file() {
        println!("{:?}", path);

        loadImage(images, path)
    } else {
        return Err("Failed to select image.".into());
    }
}

fn readBufferData<T: bytemuck::Pod>(accessor: &gltf::Accessor, buffers: &[gltf::buffer::Data]) -> Option<Vec<T>> {
    let view = accessor.view()?;
    let buffer = &buffers[view.buffer().index()];
    let data = &buffer[view.offset()..view.offset() + view.length()];

    Some(bytemuck::cast_slice(data).to_vec())
}

fn readIndices(accessor: &gltf::Accessor, buffers: &[gltf::buffer::Data]) -> Vec<u32> {
    match accessor.data_type() {
        gltf::accessor::DataType::U16 => readBufferData::<u16>(accessor, buffers).unwrap().into_iter().map(u32::from).collect(),
        gltf::accessor::DataType::U32 => readBufferData::<u32>(accessor, buffers).unwrap(),
        _ => panic!("Unsupported index format"),
    }
}

pub fn loadUserModel(meshes: ResMut<Assets<Mesh>>) -> Result<Vec<Handle<Mesh>>, String> {
    let path = FileDialog::new().add_filter("glTF model/scene", &["gltf", "glb"]).pick_file().expect("Failed to select gltf file.");

    Ok(loadModel(meshes, path).unwrap())
}

