use crate::{
    render_graph::{Node, ResourceSlotInfo, ResourceSlots},
    renderer::{BufferInfo, BufferUsage, RenderContext, RenderResourceType},
    texture::{Texture, TextureDescriptor},
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

        let width = self.descriptor.size.width as usize;
        let aligned_width = render_context.resources().get_aligned_texture_size(width);
        let format_size = self.descriptor.format.pixel_size();

        let buffer_id = render_context.resources().create_buffer(BufferInfo {
            size: self.descriptor.size.volume() * format_size,
            buffer_usage: BufferUsage::MAP_READ | BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        // this is throwing a `buffer is too small for copy` error...
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

// This code works, used to live directly in WindowTextureNode for testing...
 /*       // My additions to test copy texture to buffer and getting image out of
        // GPU
        // 
        // textures: ResourceRef
        if let Some(RenderResourceId::Texture(texture)) = output.get(WINDOW_TEXTURE) {
            let render_resource_context = render_context.resources_mut();
            let descriptor = self.descriptor;
            let width = descriptor.size.width as usize;
            let aligned_width =
                render_resource_context.get_aligned_texture_size(width);
            let format_size = descriptor.format.pixel_size();
            println!("{} {:?}", descriptor.format.pixel_info().type_size, descriptor.size);
            println!("{:?}", descriptor.format);
    
            let texture_buffer = render_resource_context.create_buffer(BufferInfo {
                size: descriptor.size.volume() * format_size,
                buffer_usage: BufferUsage::MAP_READ | BufferUsage::COPY_DST,
                mapped_at_creation: false,
            });
    
            render_context.copy_texture_to_buffer(
                texture,
                [0, 0, 0],
                0,
                texture_buffer,
                0,
                (format_size * aligned_width) as u32,
                descriptor.size,
            );

            // copy the buffer into cpu memory
    
            // remove the created buffer... for now
            render_resource_context.remove_buffer(texture_buffer);
        }*/