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
use image::load_from_memory;
use gltf;

pub fn loadImage(mut images: ResMut<Assets<Image>>, buffer: &[u8]) -> Result<Handle<Image>, String> {
    let decodedImage = load_from_memory(buffer).expect("Failed to decode image").to_rgba8();
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
}

pub fn loadModel(mut meshes: ResMut<Assets<Mesh>>, buffer: &[u8]) -> Result<Vec<Handle<Mesh>>, String> {
    let file = gltf::Gltf::from_slice(buffer).map_err(|e| format!("Failed to parse model data: {}", e))?;
    let buffers = gltf::import_buffers(&file, None, Some(file.blob.clone().unwrap())).map_err(|e| format!("Failed to get buffers from model data: {}", e))?;
    let _images = gltf::import_images(&file, None, &buffers).map_err(|e| format!("Failed to get images from model data: {}", e))?;

    let mut col: Vec<Handle<Mesh>> = vec![];

    for mesh in file.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            let positions: Vec<[f32; 3]> = reader.read_positions().ok_or("Failed to read positions")?.collect();

            let mut bevyMesh = Mesh::new(TriangleList, RenderAssetUsages::default());
            bevyMesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone());

            if let Some(normals) = reader.read_normals() {
                bevyMesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals.collect::<Vec<[f32; 3]>>());
            }

            if let Some(texCoords) = reader.read_tex_coords(0) {
                bevyMesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, texCoords.into_f32().collect::<Vec<[f32; 2]>>());
            }

            if let Some(indices) = reader.read_indices() {
                let indices = indices.into_u32().collect::<Vec<u32>>();
                bevyMesh.insert_indices(Indices::U32(indices.clone()));
            }


            col.push(meshes.add(bevyMesh));
        }
    }

    Ok(col)
}
