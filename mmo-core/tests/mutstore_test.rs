use std::{rc::Rc, cell::{RefCell}};
use mmo_core::mutation::{LocalMutation, Mutstore};

struct MyState{
    val: u32
}

struct AddMutation{
    amt: u32
}
struct SubMutation{
    a: u32,
    b: u32  // just to have different sizes
}

impl LocalMutation<MyState> for AddMutation{
    fn apply(&self, state: &mut MyState) {
        state.val += self.amt;
    }
}


impl LocalMutation<MyState> for SubMutation{
    fn apply(&self, state: &mut MyState) {
        state.val -= self.a + self.b;
    }
}
#[allow(dead_code)]
struct MutationThatNeedsDrop{
    a: String,
    b: DropTracer,
    drop_counter: Rc<RefCell<u32>>
}
impl LocalMutation<MyState> for MutationThatNeedsDrop{
    fn apply(&self, _state: &mut MyState) {
        //nop
    }
}
impl Drop for MutationThatNeedsDrop{
    fn drop(&mut self) {
        *RefCell::borrow_mut(&self.drop_counter) += 1;
    }
}
struct DropTracer{
    drop_counter: Rc<RefCell<u32>>
}

impl Drop for DropTracer{
    fn drop(&mut self) {
        *RefCell::borrow_mut(&self.drop_counter) += 1;
    }
}

#[test]
fn test_creation(){
    let mut area = Mutstore::new(512, 512);
    let mut state_a = MyState{ val: 0 };
    let mut state_b = MyState{ val: 0 };
    let desired_result = 20;
    for _ in 0 ..desired_result{
        area.create(AddMutation{ amt: 3 });
        area.create(SubMutation{ a: 1, b: 1 });
    }
    area.apply(&mut state_a);
    area.apply(&mut state_b);

    assert_eq!(state_a.val, desired_result, "state val {} should be {}", state_a.val, desired_result);
    assert_eq!(state_b.val, desired_result, "state val {} should be {}", state_b.val, desired_result);

    area.clear();

    area.apply(&mut state_a);
    area.apply(&mut state_b);

}

#[test]
fn test_clear(){
    let mut area = Mutstore::new(512, 512);
    let initial_value = 9;
    let mut state = MyState{ val: initial_value };
    let some_value = 20;
    for _ in 0 ..some_value{
        area.create(AddMutation{ amt: 3 });
    }
    area.clear();
    area.apply(&mut state);
    assert_eq!(state.val, initial_value, "state val {} should be initial_value {}", state.val, initial_value);
}

#[test]
fn test_capacity_extension(){
    let mut area = Mutstore::new(7, 7);
    let mut state = MyState{ val: 0 };
    let desired_result = 20;
    for _ in 0 ..desired_result{
        area.create(AddMutation{ amt: 1 });
    }
    area.apply(&mut state);
    assert_eq!(state.val, desired_result, "state val {} should be initial_value {}", state.val, desired_result);
}

#[test]
fn test_drop(){
    let mut area = Mutstore::new(7, 7);
    let mut state = MyState{ val: 0 };
    let desired_result = 20;
    let drop_ctr_outer = Rc::new(RefCell::new(0));
    let drop_ctr_inner = Rc::new(RefCell::new(0));
    for _ in 0 ..desired_result{
        area.create(
            MutationThatNeedsDrop{
                a: String::from("Test"),
                b: DropTracer{ drop_counter: drop_ctr_inner.clone() },
                drop_counter: drop_ctr_outer.clone(),
            }
            
        );
    }
    area.apply(&mut state);
    area.clear();
    let dropped_outer: u32 = *RefCell::borrow(&drop_ctr_outer);
    let dropped_inner: u32 = *RefCell::borrow(&drop_ctr_inner);
    assert_eq!(dropped_outer, desired_result, "did drop {} outer values but should have dropped {}", dropped_outer, desired_result);
    assert_eq!(dropped_inner, desired_result, "did drop {} outer values but should have dropped {}", dropped_inner, desired_result);
}


#[test]
fn test_instant_replay(){
    let mut area = Mutstore::new(512, 512);
    let mut state_direct = MyState{ val: 0 };
    let mut state_all = MyState{ val: 0 };

    let num_mutations = 20;

    // create one not in the replay
    // this is a bad idea if replays are used but good for testing
    area.create(AddMutation{ amt: 1 });

    area.mark_replay_position();

    for _ in 0 ..num_mutations{
        area.create(AddMutation{ amt: 1 });
    }

    area.replay_from_last_mark_and_clear_mark(&mut state_direct);

    // area.instant_replay(&mut state_direct);
    assert_eq!(state_direct.val, num_mutations, "state_direct val {} should be {} after direct replay", state_direct.val, num_mutations);

    area.replay_from_last_mark_and_clear_mark(&mut state_direct);
    assert_eq!(state_direct.val, num_mutations, "state_direct val {} should be {} after second direct replay, which should be empty", state_direct.val, num_mutations);

    area.apply(&mut state_all);
    assert_eq!(state_all.val, num_mutations+1, "state_all val {} should be {} after direct replay", state_all.val, num_mutations+1);

    area.clear();


}
