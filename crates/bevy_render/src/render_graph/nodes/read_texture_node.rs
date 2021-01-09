use crate::{
    render_graph::{Node, ResourceSlots},
    renderer::{BufferInfo, BufferUsage, RenderContext},
    texture::{Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat},
};
use bevy_asset::{Assets, Handle};
use bevy_ecs::{Resources, World};

pub struct ReadTextureNode {
    pub descriptor: TextureDescriptor,
    pub texture_handle: Handle<Texture>,
}

impl WindowTextureNode {
    pub const IN_TEXTURE: &'static str = "texture";
}

impl Node for ReadTextureNode {
    fn update(
        &mut self,
        _world: &World,
        resources: &Resources,
        render_context: &mut dyn RenderContext,
        input: &ResourceSlots,
        _output: &mut ResourceSlots,
    ) {
        const WINDOW_TEXTURE: usize = 0;

        let texture_id = input
            .get(WINDOW_TEXTURE)
            .unwrap()
            .resource
            .as_ref()
            .unwrap()
            .get_texture()
            .expect("Expected texture");

        let buffer_id = render_context.resources().create_buffer(BufferInfo {
            size: self.descriptor.size.volume(),
            buffer_usage: BufferUsage::MAP_READ | BufferUsage::COPY_DST,
            mapped_at_creation: true,
        });

        render_context.copy_texture_to_buffer(
            texture_id,
            [0, 0, 0],
            self.descriptor.mip_level_count,
            self.descriptor.size,
            buffer_id,
            self.descriptor.size.width,
            0,
        );
        let mut buffer = Vec::<u8>::with_capacity(self.descriptor.size.volume());
        render_context.resources().read_mapped_buffer(
            buffer_id,
            0..self.descriptor.size.volume() as u64,
            &mut |bytes, _| {
                buffer.extend(bytes);
            },
        );

        let texture = Texture::new(
            self.descriptor.size, 
            self.descriptor.dimension, 
            buffer, 
            self.descriptor.format);
        resources
            .get_mut::<Assets<Texture>>()
            .unwrap()
            .set_untracked(self.texture_handle.clone(), texture);
    }
}
