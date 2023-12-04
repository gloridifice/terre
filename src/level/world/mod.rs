use std::collections::HashMap;
use cgmath::{Vector2, Vector3};

type BlockPos = Vector3<i32>;
type ChuckPos = Vector2<i32>;

pub trait BlockBehaviour{

}//todo



#[derive(Clone)]
pub struct Block{
    pub registry_name: String
}

pub struct World{
    chucks: HashMap<ChuckPos, Chuck>,
}
impl World{
    pub fn new() -> Self{
        Self{
            chucks: HashMap::new()
        }
    }
}

pub struct Chuck{
    blocks: HashMap<BlockPos, Block>,
}
impl Chuck{
    pub fn new() -> Self{
        Self{
            blocks: HashMap::new(),
        }
    }
}




