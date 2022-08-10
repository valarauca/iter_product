
use std::marker::PhantomData;


pub struct WrapperVec<'a,T:'a> {
    data: Vec<SingleIter<'a,T>>,
    done: bool,
    marker: PhantomData<&'a T>,
}
impl<'a,T:'a> WrapperVec<'a,T> {

    // construct new version of itself
    pub fn new(data: Vec<&'a [T]>) -> Self {
        Self {
            data: data.into_iter().map(|slice| SingleIter::<'a,T>::new(slice)).collect(),
            done: false,
            marker: PhantomData,
        }
    }

    fn get_index_value(&mut self, index: usize, should_increment: bool) -> Option<(bool,&'a T)> {
        self.data[index].churn(should_increment)
    }
}
impl<'a,T:'a> Iterator for WrapperVec<'a,T> {
    type Item = Vec<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let mut arg = Vec::with_capacity(self.data.len());
        let mut inc_index = true;
        for i in 0..self.data.len() {
            match self.get_index_value(i, inc_index) {
                Option::None => {
                    self.done = true;
                    return None;
                }
                Option::Some((f,x)) => {
                    inc_index = f;
                    arg.push(x);
                }
            }
        }
        self.done = inc_index;
        Some(arg)
    }
}

pub struct Wrapper<'a, T:'a, const COUNT: usize> {
    data: [SingleIter<'a,T>;COUNT],
    done: bool,
    marker: PhantomData<&'a T>,
}
impl<'a, T:'a, const COUNT: usize> Wrapper<'a,T,COUNT> {

    // construct new version of itself
    pub fn new(data: [&'a [T];COUNT ]) -> Self {

        #[allow(deprecated)]
        let mut iter: [SingleIter<'a,T>; COUNT] = unsafe { std::mem::uninitialized() };
        for i in 0..COUNT {
           iter[i] = SingleIter::<'a,T>::new(data[i])
        }
        Self {
            data: iter,
            done: false,
            marker: PhantomData,
        }
    }

    fn get_index_value(&mut self, index: usize, should_increment: bool) -> Option<(bool,&'a T)> {
        self.data[index].churn(should_increment)
    }
}

impl<'a,T:'a,const COUNT: usize> Iterator for Wrapper<'a,T,COUNT> {
    type Item = [&'a T;COUNT];

    fn next(&mut self) -> Option<Self::Item> {
       
        if self.done {
            return None;
        }

        #[allow(deprecated)]
        let mut arg: Self::Item = unsafe { std::mem::uninitialized::<Self::Item>() }; 

        let mut inc_index = true;
        //self.is_fresh = true;
        for index in 0..COUNT {
            match self.get_index_value(index, inc_index) {
                Option::None => {
                    self.done = true;
                    return None;
                }
                Option::Some((f,x)) => {
                    inc_index = f;
                    arg[index] = x;
                }
            }
        }
        self.done = inc_index;
        Some(arg)
    }
}
#[test]
fn test_wrapper_product_1() {
    const A: &'static [usize] = &[1];
    const B: &'static [usize] = &[2];
    const C: &'static [usize] = &[3,4,5];
    const D: &'static [usize] = &[6];

    let mut dut = Wrapper::new([A,B,C,D]);

    assert_eq!(dut.next(), Option::Some([&1, &2, &3, &6]));
    assert_eq!(dut.next(), Option::Some([&1, &2, &4, &6]));
    assert_eq!(dut.next(), Option::Some([&1, &2, &5, &6]));
    assert_eq!(dut.next(), Option::None);
}

#[test]
fn test_wrapper_product_2() {
    const A: &'static [usize] = &[1,2];
    const B: &'static [usize] = &[3,4];
    const C: &'static [usize] = &[5];

    let mut dut = Wrapper::new([A,B,C]);

    assert_eq!(dut.next(), Option::Some([&1, &3, &5]));
    assert_eq!(dut.next(), Option::Some([&2, &3, &5]));
    assert_eq!(dut.next(), Option::Some([&1, &4, &5]));
    assert_eq!(dut.next(), Option::Some([&2, &4, &5]));
    assert_eq!(dut.next(), Option::None);
}

#[test]
fn test_wrapper_product_3() {
    const A: &'static [usize] = &[1,2];
    const B: &'static [usize] = &[3,4];
    const C: &'static [usize] = &[];

    let mut dut = Wrapper::new([A,B,C]);

    assert_eq!(dut.next(), Option::None);
}

struct SingleIter<'a,T: 'a> {
    inner: &'a [T],
    pos: usize,
}
impl<'a,T:'a> SingleIter<'a,T> {

    fn new(arg: &'a [T]) -> Self {
        Self {
            inner: arg,
            pos: 0,
        }
    }

    fn churn(&mut self, should_increment: bool) -> Option<(bool,&'a T)> {
        match self.get_current() {
            Option::None => None,
            Option::Some(x) => {
                if should_increment {
                    self.inc();
                }
                let increment_next = should_increment && self.has_terminated();
                if increment_next {
                    self.restart();
                }
                Some((increment_next,x))
            }
        }
    }

    fn restart(&mut self) {
        self.pos = 0;
    }
    
    fn inc(&mut self) {
        self.pos += 1;
    }

    fn has_terminated(&self) -> bool {
        self.pos >= self.inner.len()
    }

    fn get_current(&self) -> Option<&'a T> {
        if !self.has_terminated() {
            Some(&self.inner[self.pos])
        } else {
            None
        }
    }
}
