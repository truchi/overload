#![allow(unused)]

fn main() {}

pub struct Overloaded;

mod a {
    use super::*;

    impl Overloaded {
        fn lala<T: LolArgs>(self, args: T) -> T::Output {
            args.lala(self)
        }
    }

    pub trait LolArgs {
        type Output;

        fn lala(self, overloaded: Overloaded) -> Self::Output;
    }

    impl LolArgs for u8 {
        type Output = u8;

        fn lala<'z>(self, o: Overloaded) -> u8 {
            o.lala_1(self)
        }
    }

    impl<'b> LolArgs for &'b u8 {
        type Output = &'b u8;

        fn lala(self, o: Overloaded) -> &'b u8 {
            o.lala_2(self)
        }
    }

    impl Overloaded {
        pub fn lala_1<'z>(self, a: u8) -> u8 {
            a
        }

        pub fn lala_2<'a>(self, a: &'a u8) -> &'a u8 {
            a
        }
    }
}

mod b {
    use super::*;

    impl Overloaded {
        fn lol<__T: LolArgs>(&self, args: __T) -> __T::Output<'_> {
            args.lol(self)
        }
    }

    pub trait LolArgs {
        type Output<'s>;

        fn lol<'s>(self, overloaded: &'s Overloaded) -> Self::Output<'s>;
    }

    impl LolArgs for u8 {
        type Output<'s> = u8;

        fn lol<'s>(self, o: &'s Overloaded) -> u8 {
            o.lol_1(self)
        }
    }

    impl<'b> LolArgs for &'b u8 {
        type Output<'__self> = &'b u8;

        fn lol<'__self>(self, __self: &'__self Overloaded) -> &'b u8 {
            __self.lol_2(self)
        }
    }

    // impl<'a, 'b> LolArgs for (&'a u16, &'b u16) {
    //     type Output<'s> = &'a u16;

    //     fn lol<'s>(self, o: &'s Overloaded) -> &'a u16 {
    //         o.lol_3(self.0, self.1)
    //     }
    // }

    impl Overloaded {
        pub fn lol_1(&self, a: u8) -> u8 {
            a
        }

        pub fn lol_2<'a>(&self, a: &'a u8) -> &'a u8 {
            a
        }

        // pub fn lol_3<'a, 'b>(&'a self, a: &'a u16, b: &'b u16) -> &'a u16 {
        //     a
        // }
    }
}

mod input {
    pub struct Add(u8);

    #[overload::overload]
    impl Add {
        const CONST: u8 = 0;

        pub fn add(&self, a: u8) -> u16 {
            todo!()
        }

        pub fn add(&self, a: u8, b: u8) -> u16 {
            self.0 as u16 + a as u16 + b as u16
        }

        pub fn add(&self, a: u16, b: u16) -> u32 {
            self.0 as u32 + a as u32 + b as u32
        }
    }

    fn test(add: &Add) {
        add.add((0u8, 1u8));
        add.add((0u16, 1u16));
    }
}

mod output {
    pub struct Add(u8);

    impl Add {
        const CONST: u8 = 0;

        pub fn add<T: AddArgs>(&self, args: T) -> T::Output {
            args.add(self)
        }

        #[doc(hidden)]
        pub fn __add_3333(&self, a: u8) -> u16 {
            todo!()
        }

        #[doc(hidden)]
        pub fn __add_0(&self, a: u8, b: u8) -> u16 {
            self.0 as u16 + a as u16 + b as u16
        }

        #[doc(hidden)]
        pub fn __add_1(&self, a: u16, b: u16) -> u32 {
            self.0 as u32 + a as u32 + b as u32
        }
    }

    trait AddArgs {
        type Output;

        fn add(self, add: &Add) -> Self::Output;
    }

    impl AddArgs for u8 {
        type Output = u16;

        fn add(self, add: &Add) -> Self::Output {
            add.__add_3333(self.0)
        }
    }

    impl AddArgs for (u8, u8) {
        type Output = u16;

        fn add(self, add: &Add) -> Self::Output {
            add.__add_0(self.0, self.1)
        }
    }

    impl AddArgs for (u16, u16) {
        type Output = u32;

        fn add(self, add: &Add) -> Self::Output {
            add.__add_1(self.0, self.1)
        }
    }

    fn test(add: &Add) {
        add.add((0u8, 1u8));
        add.add((0u16, 1u16));
    }
}

// /// # Some doc
// #[overload::overload(LolArgs)]
// // #[async_trait::async_trait]
// #[doc(hidden)]
// impl Overloaded {
//     /// # A function
//     pub(crate) unsafe fn lol(&mut self, a: u8, b: usize) -> Result<Self, ()> {}

//     /// # Another function
//     pub(crate) unsafe fn lol<T: Clone>(&mut self, c: i8, d: T) -> Option<String> {}

//     /// # A function
//     pub(crate) unsafe fn lol(&mut self, a: u8, b: usize) {}

//     /// # A function
//     pub(crate) unsafe fn lol(&mut self, a: u8, b: usize) -> () {}
// }
