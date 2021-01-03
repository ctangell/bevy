use crate::{
    render_graph::{Node, ResourceSlots},
    renderer::{BufferInfo, BufferUsage, RenderContext},
    texture::{Extent3d, Texture, TextureDimension, TextureFormat},
};
use bevy_asset::{Assets, Handle};
use bevy_ecs::{Resources, World};

pub struct ReadTextureNode {
    read_slot: String,
    origin: [u32; 3],
    mip_level: u32,
    size: Extent3d,
    dimension: TextureDimension,
    texture_handle: Handle<Texture>,
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
        let texture_id = input
            .get_slot(self.read_slot.to_owned())
            .unwrap()
            .resource
            .as_ref()
            .unwrap()
            .get_texture()
            .expect("Expected texture");

        let buffer_id = render_context.resources().create_buffer(BufferInfo {
            size: self.size.volume(),
            buffer_usage: BufferUsage::MAP_READ | BufferUsage::COPY_DST,
            mapped_at_creation: true,
        });

        render_context.copy_texture_to_buffer(
            texture_id,
            self.origin,
            self.mip_level,
            self.size,
            buffer_id,
            self.size.width,
            0,
        );
        let mut buffer = Vec::<u8>::with_capacity(self.size.volume());
        render_context.resources().read_mapped_buffer(
            buffer_id,
            0..self.size.volume() as u64,
            &mut |bytes, _| {
                buffer.extend(bytes);
            },
        );

        let texture = Texture::new(self.size, self.dimension, buffer, TextureFormat::default());
        resources
            .get_mut::<Assets<Texture>>()
            .unwrap()
            .set_untracked(self.texture_handle.clone(), texture);
    }
}
