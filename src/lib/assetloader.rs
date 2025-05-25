use std::{collections::HashMap, io::Cursor, path::Path};

use bevy::{
    prelude::*, render::{
        mesh::PrimitiveTopology::TriangleList,
        render_asset::RenderAssetUsages,
        render_resource::{
            Extent3d,
            TextureDescriptor, 
            TextureDimension, 
            TextureFormat, 
            TextureUsages,
        },
    }
};
use bevy_mesh::Indices;
use gltf_json::{image::MimeType, Index};
use image::{load_from_memory, DynamicImage, ImageBuffer, ImageFormat, Rgb};
use gltf::{self, image::Source};

use crate::wasm::{data::MemoryDir, definitions::consoleLog};

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

fn loadTexture(source: Source) -> Option<usize> {
    match source {
        Source::View { view, mime_type: _ } => {
            Some(view.buffer().index())
        },
        Source::Uri { uri: _, mime_type: _ } => None,
    }
}

fn u8tou16(buffer: &[u8]) -> Result<Vec<u16>, String> {
    if buffer.len() % 2 != 0 {
        return Err("Buffer length must be a multiple of 2 for u8 to u16 conversion".into());
    }
    let mut out = Vec::with_capacity(buffer.len() / 2);
    
    for chunk in buffer.chunks_exact(2) {
        let value = u16::from_le_bytes([chunk[0], chunk[1]]);
        out.push(value);
    }

    Ok(out)
}

fn glbTo8Bit(buffer: &[u8]) -> Result<Vec<u8>, String> {
    let glb = gltf::Glb::from_slice(buffer).map_err(|e| format!("Failed to split GLB: {}", e))?;
    let mut root: gltf_json::Root = serde_json::from_slice(&glb.json).map_err(|e| format!("JSON parse error: {}", e))?;

    let doc = gltf::Gltf::from_slice(buffer).map_err(|e| format!("Failed to parse model data: {}", e))?;
    let buffers = gltf::import_buffers(&doc, None, glb.bin.clone().map(|c| c.to_vec())).map_err(|e| format!("Failed to get buffers from model data: {}", e))?;
    let images = gltf::import_images(&doc, None, &buffers).map_err(|e| format!("Failed to get images from model data: {}", e))?;

    let mut bin = glb.bin.map(|c| c.into_owned()).unwrap_or_else(Vec::new);
    let baseViewCount = root.buffer_views.len();

    for (i, image) in images.iter().enumerate() {
        consoleLog(&format!("Image index {}", i));
        // let dynImg = load_from_memory(&image.pixels).map_err(|e| format!("Failed to decode image: {}", e))?;
        let dynImg = match image.format {
            gltf::image::Format::R16G16B16 => {
                consoleLog(&format!("16-bit, height: {}, width: {}, pixel length: {}", image.height, image.width, image.pixels.len()));
                // let u16Pixels: Vec<u16> = cast_vec(image.pixels.clone());
                let u16Pixels = u8tou16(&image.pixels).map_err(|e| format!("Failed to convert 8-bit to 16-bit: {}", e))?;
                let buf: ImageBuffer<Rgb<u16>, _> = ImageBuffer::from_raw(image.width, image.height, u16Pixels).ok_or("Failed to create ImageBuffer (from 16-bit rgb)")?;
                DynamicImage::ImageRgb16(buf)
            },
            gltf::image::Format::R8G8B8 => {
                let buf: ImageBuffer<Rgb<u8>, _> = ImageBuffer::from_raw(image.width, image.height, image.pixels.clone()).unwrap();
                DynamicImage::ImageRgb8(buf)
            },
            other => {
                return Err(format!("Unsupported pixel encoding: {:?}", other));
            }
        };

        let mut data = vec![];
        dynImg.to_rgb8().write_to(&mut Cursor::new(&mut data), ImageFormat::Jpeg).map_err(|e| format!("JPEG encode error: {}", e))?;

        let offset = ((bin.len() + 3) & !3) as u32;
        bin.resize(offset as usize, 0);
        bin.extend_from_slice(&data);

        root.buffer_views.push(gltf_json::buffer::View {
            buffer: Index::new(0),
            byte_offset: Some((offset as u64).into()),
            byte_length: (data.len() as u64).into(),
            byte_stride: None,
            target: None,
            name: None,
            extensions: None,
            extras: None,
        });

        let imageJson = &mut root.images[i];
        imageJson.uri = None;
        imageJson.buffer_view = Some(Index::new((baseViewCount + i) as u32));
        imageJson.mime_type = Some(MimeType("image/jpeg".into()));
    }

    let newJson = serde_json::to_vec(&root).map_err(|e| format!("Failed to serialize JSON: {}", e))?;

    let outGlb = gltf::Glb {
        header: glb.header,
        json: newJson.into(),
        bin: Some(bin.into()),
    };

    outGlb.to_vec().map_err(|e| format!("Failed to convert GLB to Vec<u8>: {}", e))
}

pub fn loadModelV2(assetServer: ResMut<AssetServer>, _assetName: String, memDir: ResMut<MemoryDir>, buffer: Vec<u8>) -> Handle<Scene> {
    let bit8 = glbTo8Bit(&buffer).expect("Failed to convert GLB to 8-bit");

    memDir.dir.insert_asset(Path::new("test.glb"), bit8);
    assetServer.load(GltfAssetLabel::Scene(0).from_asset("memory://test.glb"))
}

pub fn loadModel(mut meshes: ResMut<Assets<Mesh>>, mut images: ResMut<Assets<Image>>, mut materials: ResMut<Assets<StandardMaterial>>, buffer: &[u8]) -> Result<Vec<(Handle<Mesh>, Handle<StandardMaterial>)>, String> {
    let bit8 = glbTo8Bit(buffer).map_err(|e| format!("Failed to convert GLB to 8-bit: {}", e))?;
    
    let file = gltf::Gltf::from_slice(&bit8).map_err(|e| format!("Failed to parse model data: {}", e))?;
    let buffers = gltf::import_buffers(&file, None, Some(file.blob.clone().unwrap())).map_err(|e| format!("Failed to get buffers from model data: {}", e))?;
    let _images = gltf::import_images(&file, None, &buffers).map_err(|e| format!("Failed to get images from model data: {}", e))?;

    let mut col: Vec<(Handle<Mesh>, Handle<StandardMaterial>)> = vec![];
    let mut mcol: Vec<Handle<StandardMaterial>> = vec![];

    for material in file.materials() {
        let pbr = material.pbr_metallic_roughness();

        let mut imageMap: HashMap<&str, Handle<Image>> = HashMap::new();

        if let Some(texInfo) = pbr.base_color_texture() {
            // let index = loadTexture(texInfo.texture().source().source()).unwrap();
            // let image = _images.get(index).unwrap();
            let index = texInfo.texture().source().index();
            consoleLog(format!("Index: {}, Image indices: {}", index, _images.len()).as_str());
            consoleLog(format!("image: {:?}", _images.get(index)).as_str());
            consoleLog(format!("imagev2: {:?}", &_images[index]).as_str());
            let image = _images.get(index).unwrap();

            imageMap.insert("colour", images.add(
                Image::new(
                    Extent3d { width: image.width, height: image.height, depth_or_array_layers: 1 },
                    TextureDimension::D2,
                    image.pixels.clone(),
                    TextureFormat::Rgba8UnormSrgb,
                    RenderAssetUsages::all(),
                ),
            ));
        }

        // if let Some(normal) = material.normal_texture() {
        //     let index = loadTexture(normal.texture().source().source()).unwrap();
        //     let image = _images.get(index).unwrap();

        //     imageMap.insert("normal", images.add(
        //         Image::new(
        //             Extent3d { width: image.width, height: image.height, depth_or_array_layers: 1 },
        //             TextureDimension::D2,
        //             image.pixels.clone(),
        //             TextureFormat::Rgba8UnormSrgb,
        //             RenderAssetUsages::all(),
        //         ),
        //     ));
        // }

        // if let Some (rm) = pbr.metallic_roughness_texture() {
        //     let index = loadTexture(rm.texture().source().source()).unwrap();
        //     let image = _images.get(index).unwrap();

        //     imageMap.insert("rm", images.add(
        //         Image::new(
        //             Extent3d { width: image.width, height: image.height, depth_or_array_layers: 1 },
        //             TextureDimension::D2,
        //             image.pixels.clone(),
        //             TextureFormat::Rgba8UnormSrgb,
        //             RenderAssetUsages::all(),
        //         ),
        //     ));
        // }
        
        mcol.push(materials.add(StandardMaterial {
            base_color_texture: imageMap.get("colour").cloned(),
            normal_map_texture: imageMap.get("normal").cloned(),
            metallic_roughness_texture: imageMap.get("rm").cloned(),
            ..default()
        }));
    }

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


            // col.push(meshes.add(bevyMesh));
            consoleLog(format!("Index: {}, Mcol Indices: {}", primitive.material().index().unwrap_or(0), mcol.len()).as_str());
            let matHandle = mcol.get(primitive.material().index().unwrap_or(0)).cloned().unwrap_or_else(|| mcol[0].clone());
            consoleLog("Past matHandle definition");
            col.push((
                meshes.add(bevyMesh),
                matHandle,
            ));
        }
    }

    Ok(col)
}
