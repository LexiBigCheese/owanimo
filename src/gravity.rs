use crate::Board;

pub trait GravityBoard {
    fn fall(&mut self) -> bool;
}

pub trait AutoGravityBoard: Board {
    fn is_air(&self, handle: Self::Handle) -> bool;
    fn mutate_columns(&mut self, mutater: impl FnMut(&Self, &mut [Self::Handle]));
}

impl<T: AutoGravityBoard> GravityBoard for T {
    fn fall(&mut self) -> bool {
        let mut did_fall = false;
        self.mutate_columns(|this, col| {
            let mut index = 0;
            'outer: loop {
                let cursor_a = index;
                //cursor starts at bottom, assumes itself to be air,
                while this.is_air(col[index]) {
                    index += 1;
                    if index == col.len() {
                        break 'outer;
                    };
                }
                let cursor_b = index;
                //now find how many are not
                while !this.is_air(col[index]) {
                    index += 1;
                    if index == col.len() {
                        break;
                    }
                }
                if cursor_b != cursor_a {
                    did_fall = true;
                }
                //then rotate cursor_a..index leftwards to push the air bubbles to the top and repeat
                col[cursor_a..index].rotate_left(cursor_b - cursor_a);
                //and rotate the index too
                index -= cursor_b - cursor_a;
                if index == col.len() {
                    break;
                }
            }
        });
        did_fall
    }
}
