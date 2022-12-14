use bevy::{
    asset::LoadState,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{MeshVertexAttribute, MeshVertexBufferLayout},
        render_resource::{
            AsBindGroup, CompareFunction, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
            VertexFormat,
        },
    },
};
pub struct ChunkTexture(pub Handle<Image>);

pub fn load_chunk_texture(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(ChunkTexture(server.load("array_test.png")));
}

pub fn create_array_texture(
    asset_server: Res<AssetServer>,
    texture: Res<ChunkTexture>,
    mut images: ResMut<Assets<Image>>,
) {
    while asset_server.get_load_state(texture.0.clone()) != LoadState::Loaded {
        panic!("waiting on load, please fix this");
    }
    let image = images.get_mut(&texture.0).unwrap();
    if image.texture_descriptor.size.depth_or_array_layers != 1 {
        return;
    }

    // Create a new array texture asset from the loaded texture.
    let array_layers = image.texture_descriptor.size.height / image.texture_descriptor.size.width;
    image.reinterpret_stacked_2d_as_array(array_layers);
}

pub const CUSTOM_UV: MeshVertexAttribute = MeshVertexAttribute::new("CustomUV", 52894552143, VertexFormat::Uint8x2);

pub const CUSTOM_NORMAL: MeshVertexAttribute =
    MeshVertexAttribute::new("CutsomNormal", 1374029579328, VertexFormat::Uint8x2);

pub const ATTRIBUTE_TEXTURE_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("TextureIndex", 15092354854, VertexFormat::Uint32);

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct CustomMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub textures: Handle<Image>,
}

impl Material for CustomMaterial {
    fn vertex_shader() -> ShaderRef {
        "custom_material.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "custom_material.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            CUSTOM_NORMAL.at_shader_location(1),
            CUSTOM_UV.at_shader_location(2),
            ATTRIBUTE_TEXTURE_INDEX.at_shader_location(3),
        ]);
        descriptor.depth_stencil.as_mut().unwrap().depth_write_enabled = true;
        //Ugh FIXME Transparent faces need to be ordered or seperate mesh
        descriptor.depth_stencil.as_mut().unwrap().depth_compare = CompareFunction::GreaterEqual;
        let vertex_layout = vertex_layout.unwrap();
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
