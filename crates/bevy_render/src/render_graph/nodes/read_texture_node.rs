use crate::{
    render_graph::{Node, ResourceSlotInfo, ResourceSlots},
    renderer::{BufferInfo, BufferUsage, RenderContext, RenderResourceId, RenderResourceType},
    texture::{Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat},
};
use bevy_asset::{Assets, Handle};
use bevy_ecs::{Resources, World};
use std::borrow::Cow;

pub struct ReadTextureNode {
    pub descriptor: TextureDescriptor,
    pub texture_handle: Handle<Texture>,
}

impl ReadTextureNode {
    pub const IN_TEXTURE: &'static str = "texture";
}

impl Node for ReadTextureNode {
    fn input(&self) -> &[ResourceSlotInfo] {
        static INPUT: &[ResourceSlotInfo] = &[ResourceSlotInfo {
            name: Cow::Borrowed(ReadTextureNode::IN_TEXTURE),
            resource_type: RenderResourceType::Texture,
        }];
        INPUT
    }

    fn update(
        &mut self,
        _world: &World,
        resources: &Resources,
        render_context: &mut dyn RenderContext,
        input: &ResourceSlots,
        _output: &mut ResourceSlots,
    ) {
        const INPUT_TEXTURE: usize = 0;

        let texture_id = input.get(INPUT_TEXTURE).unwrap().get_texture().unwrap();

        println!("{:?}", texture_id);

        let width = self.descriptor.size.width as usize;
        let aligned_width = render_context.resources().get_aligned_texture_size(width);
        let format_size = self.descriptor.format.pixel_size();

        let buffer_id = render_context.resources().create_buffer(BufferInfo {
            size: self.descriptor.size.volume() * format_size,
            buffer_usage: BufferUsage::MAP_READ | BufferUsage::COPY_DST,
            mapped_at_creation: true,
        });

        render_context.copy_texture_to_buffer(
            texture_id,
            [0, 0, 0],
            self.descriptor.mip_level_count,
            buffer_id,
            0,
            (format_size * aligned_width) as u32,
            self.descriptor.size,
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

        render_context.resources().remove_buffer(buffer_id);
    }
}