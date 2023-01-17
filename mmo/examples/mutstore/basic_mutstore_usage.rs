
use mmo_core::mutation::{Mutstore, LocalMutation};

struct MyState{
    val: u32
}

struct AddMutation{
    amt: u32
}
impl LocalMutation<MyState> for AddMutation{
    fn apply(&self, state: &mut MyState) {
        state.val += self.amt;
    }
}

pub fn main() {

    let size : usize = 512*1024*1024;

    let num_mutations = 100_000_000;

    let mut area = Mutstore::new(size, num_mutations);
    let mut state = MyState{ val: 0 };

    append_mutations(num_mutations, &mut area);
    apply_to_existing_state(&mut area, &mut state);    
    println!("result value {}",state.val)
}


fn append_mutations(num_mutations: usize, area: &mut Mutstore<MyState>){
    for _ in 0 ..num_mutations{
        area.create(AddMutation{ amt: 1 });
    }
}

fn apply_to_existing_state(area: &mut Mutstore<MyState>, state: &mut MyState){
    area.apply(state);
} 
