use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{MeshVertexAttribute, MeshVertexBufferLayout},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError, VertexFormat,
        },
    },
};

pub const CUSTOM_UV: MeshVertexAttribute = MeshVertexAttribute::new("CustomUV", 52894552143, VertexFormat::Uint8x2);

pub const CUSTOM_NORMAL: MeshVertexAttribute =
    MeshVertexAttribute::new("CutsomNormal", 1374029579328, VertexFormat::Uint8x2);

pub const ATTRIBUTE_TEXTURE_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("TextureIndex", 15092354854, VertexFormat::Uint32);

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct CustomMaterial {
    #[texture(0)]
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

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            //FIXME we don't need this level of detail for normals
            CUSTOM_NORMAL.at_shader_location(1),
            //FIXME we dont need uvs, just a vertex id
            CUSTOM_UV.at_shader_location(2),
            ATTRIBUTE_TEXTURE_INDEX.at_shader_location(3),
        ]);
        let vertex_layout = vertex_layout.unwrap();
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
