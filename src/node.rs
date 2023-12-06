use wgpu::{Buffer, Device};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::instance::Instance;
use crate::model::Model;

pub struct Node{
    pub parent: u32,
    pub model: Model,
    instances: Vec<Instance>,
    instance_buffer: Buffer,
}

impl Node{
    pub fn new(device: &Device, parent: u32, model: Model, instances: Vec<Instance>) -> Self{
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(
            &BufferInitDescriptor{
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        Self{
            parent,
            model,
            instances,
            instance_buffer
        }
    }

    pub fn instances_len(&self) -> usize{
       self.instances.len()
    }
    pub fn instance_buffer(&self) -> &Buffer{
        &self.instance_buffer
    }
}

