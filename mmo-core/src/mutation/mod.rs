mod mutstore;

pub use mutstore::Mutstore;

pub trait LocalMutation<T>{
    fn apply(&self, state: &mut T);
} 