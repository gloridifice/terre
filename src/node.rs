use crate::instance::Instance;
use crate::model::Model;

pub struct Node{
    pub parent: u32,
    pub model: Model,
    pub instances: Vec<Instance>,
}

