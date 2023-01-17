use std::{pin::Pin, ptr::{NonNull, self}, marker::PhantomPinned};

use bumpalo::Bump;

use super::LocalMutation;


pub struct Mutstore<T>{
    inner: Pin<Box<MutstoreInner<T>>>,
}
impl<T> Mutstore<T>{
    pub fn new(area_capacity: usize, vec_capacity: usize) -> Self{
        Mutstore{
            inner: MutstoreInner::new(area_capacity, vec_capacity),
        }
    }
    
    pub fn create<E: 'static + LocalMutation<T>>(&mut self, element: E ){
        let inner = self.get_inner_mut();
        inner.create(element)
    }

    pub fn mark_replay_position(&mut self ){
        let inner = self.get_inner_mut();
        inner.mark_position()
    }
    pub fn replay_from_last_mark_and_clear_mark(&mut self , state: &mut T){
        let inner = self.get_inner_mut();
        inner.replay_from_mark(state);
        inner.clear_mark_position()
    }

    pub fn apply(&self, state: &mut T){
        self.inner.apply(state)
    }

    pub fn clear(&mut self){
        let inner = self.get_inner_mut();
        inner.clear()
    }

    fn get_inner_mut(&mut self) -> &mut MutstoreInner<T>{
        unsafe {
            let mut_ref: Pin<&mut MutstoreInner<T>> = Pin::as_mut(&mut self.inner);
            Pin::get_unchecked_mut(mut_ref)
        }
    }

}


struct MutstoreInner<T>{
    bump: Bump,
    muts: Vec<NonNull< dyn LocalMutation<T>>>,
    _pin: PhantomPinned,
    replay_position: Option<usize>
}

impl<T> MutstoreInner<T>{
    fn new(area_capacity: usize, vec_capacity: usize) -> Pin<Box<Self>>{
        Box::pin(MutstoreInner{
            bump: Bump::with_capacity(area_capacity),
            muts: Vec::with_capacity(vec_capacity),
            _pin: PhantomPinned,
            replay_position: None
        })
    }

    fn create<E:'static + LocalMutation<T>>(&mut self, element: E )
    {
        let s = self.bump.alloc(element);
        self.muts.push(NonNull::from(s));
    }

    fn mark_position(&mut self){
        self.replay_position = Some(self.muts.len());
    }
    fn clear_mark_position(&mut self){
        self.replay_position = None;
    }

    fn replay_from_mark(&mut self,  state: &mut T){
        if let Some(replay_position) = self.replay_position{
            self.apply_subset(state, &self.muts[replay_position..])
        }
    }


    fn clear(&mut self){
        self.muts.iter().for_each(|i|
            unsafe{
                let ptr = i.as_ptr();
                ptr::drop_in_place(ptr)
            });
        self.muts.clear();
        self.bump.reset();
    }
    
    fn apply(&self, state: &mut T){
        self.apply_subset(state, &self.muts)
    }
    fn apply_subset(&self, state: &mut T, muts: &[NonNull< dyn LocalMutation<T>>] ){
        for m in muts{
            // SAFETY Hope for the best
            let r =   unsafe{ m.as_ref() };
            r.apply(state)
        }
    }
}





#[cfg(test)]
mod test{


    use std::{pin::Pin, thread};

    use crate::mutation::LocalMutation;
    use super::{Mutstore, MutstoreInner};

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
    macro_rules! pinny {
        ( $x:expr ) => {
            {
                let mut_ref: Pin<&mut MutstoreInner<MyState>> = Pin::as_mut(&mut $x);
                unsafe{ Pin::get_unchecked_mut(mut_ref) as *const _ as usize}
            }
        };
    }

    #[test]
    fn test_move_around_handle_does_not_move_pin(){

        let _result = thread::spawn(||{
            let mut mutations = Mutstore::new(512, 512);
            let initial_value = 0;
            let mut state = MyState{ val: initial_value };
    
    
            // gather and check initial pointer info
            let ptr_initial_pin = &mutations.inner._pin as *const _ as usize;
            let ptr_initial = &mutations.inner as *const _ as usize;
            let ptr_initial_unchecked = pinny!(mutations.inner);
            assert_eq!(ptr_initial_pin, ptr_initial, "state val {} should be expected_value {}", ptr_initial_pin, ptr_initial);
            assert_eq!(ptr_initial_unchecked, ptr_initial, "state val {} should be expected_value {}", ptr_initial_unchecked, ptr_initial);
    
            // use the data structure and move it into a function
            mutations.create(AddMutation{ amt: 1 });
            let (mut mutations, ptr_fn) = move_around_mutstore_handle(mutations);
            mutations.create(AddMutation{ amt: 1 });
    
    
            // check result is ok
            let expected_value = 3* 1;
            mutations.apply(&mut state);
            assert_eq!(state.val, expected_value, "state val {} should be expected_value {}", state.val, expected_value);
    
            // gather and check pointer info after we got back the data structure
            let ptr_post_unchecked = pinny!(mutations.inner);
            assert_eq!(ptr_initial, ptr_fn, "all pointers should match {:x} != {:x}",ptr_initial, ptr_fn);
            assert_eq!(ptr_initial, ptr_post_unchecked, "all pointers should match {:x} != {:x}",ptr_initial, ptr_post_unchecked);
       
        }).join();

    }
    
    fn move_around_mutstore_handle(mut mutstore: Mutstore<MyState>) -> (Mutstore<MyState>, usize){        
        let ptr_fn = pinny!(mutstore.inner);
        let m = AddMutation{ amt: 1 };
        mutstore.create(m);
        (mutstore, ptr_fn)
    }

}