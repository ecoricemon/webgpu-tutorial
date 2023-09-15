pub trait Number {
    type Output;
    fn zero() -> Self::Output;
    fn one() -> Self::Output;
    fn max() -> Self::Output;
    fn _sqrt(self) -> Self::Output;
    fn _acos(self) -> Self::Output;
}

impl Number for u8 {
    type Output = u8;

    #[inline]
    fn zero() -> Self::Output {
        0
    }

    #[inline]
    fn one() -> Self::Output {
        1
    }

    #[inline]
    fn max() -> Self::Output {
        u8::MAX
    }

    #[inline]
    fn _sqrt(self) -> Self::Output {
        panic!("Oops! There's no sqrt() for u8")
    }

    #[inline]
    fn _acos(self) -> Self::Output {
        panic!("Oops! There's no acos() for u8")
    }
}

impl Number for f32 {
    type Output = f32;

    #[inline]
    fn zero() -> Self::Output {
        0.0
    }

    #[inline]
    fn one() -> Self::Output {
        1.0
    }

    #[inline]
    fn max() -> Self::Output {
        f32::MAX
    }

    #[inline]
    fn _sqrt(self) -> Self::Output {
        self.sqrt()
    }

    #[inline]
    fn _acos(self) -> Self::Output {
        self.acos()
    }
}

#[derive(Debug, PartialEq, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(transparent)]
pub struct Vector<T, const D: usize>(pub [T; D]);

macro_rules! strip_first_op {
    (+ $($tail:tt)+) => {
        $($tail)+
    };
    (- $($tail:tt)+) => {
        $($tail)+
    };
    (* $($tail:tt)+) => {
        $($tail)+
    };
    (/ $($tail:tt)+) => {
        $($tail)+
    };
}

#[macro_export]
macro_rules! impl_vector {
    ($d:expr, $({$field:ident:$index:tt}),+) => {
        impl<T> Vector<T, $d>
        where
            T: Copy
                + std::ops::Add<Output = T>
                + std::ops::Sub<Output = T>
                + std::ops::Mul<Output = T>
                + std::ops::Div<Output = T>
                + std::ops::DivAssign
                + Number<Output = T>
                + PartialEq
        {
            #[inline]
            pub fn new($($field: T),+) -> Self {
                Self([$($field),+])
            }

            $(
                /// Getter
                #[inline]
                pub fn $field(&self) -> T {
                    self.0[$index]
                }

                paste::item! {
                    /// Setter
                    #[inline]
                    pub fn [<set_ $field>](&mut self, v: T) {
                        self.0[$index] = v;
                    }
                }
            )+

            #[inline]
            pub fn iter(&self) -> core::slice::Iter<T> {
                self.0.iter()
            }

            #[inline]
            pub fn norm_l2(&self) -> T {
                strip_first_op!(
                    $(+ self.0[$index] * self.0[$index])+
                )._sqrt()
            }

            #[inline]
            pub fn normalize(&mut self) {
                let norm = self.norm_l2();
                match norm != T::zero() {
                    true => {$(self.0[$index] /= norm);+}
                    false => (),
                }
            }

            #[inline]
            pub fn dist(self, rhs: Self) -> T {
                (self - rhs).norm_l2()
            }

            #[inline]
            pub fn dot_product(self, rhs: Self) -> T {
                strip_first_op!(
                    $(+ self.0[$index] * rhs.0[$index])+
                )
            }

            #[inline]
            pub fn cross_product(self, rhs: Self) -> Self {
                // todo!("Generalize");
                debug_assert!($d == 3);
                Self([
                    self.0[1] * rhs.0[2] - self.0[2] * rhs.0[1],
                    self.0[2] * rhs.0[0] - self.0[0] * rhs.0[2],
                    self.0[0] * rhs.0[1] - self.0[1] * rhs.0[0],
                ])
            }

            #[inline]
            pub fn angle(self, rhs: Self) -> T {
                (self.dot_product(rhs) / self.norm_l2() / rhs.norm_l2())._acos()
            }
        }

        impl<T: Copy> From<T> for Vector<T, $d> {
            #[inline]
            fn from(value: T) -> Self {
                Self([value; $d])
            }
        }


        impl<T: Copy> From<&[T]> for Vector<T, $d> {
            #[inline]
            fn from(value: &[T]) -> Self {
                Self(value.try_into().expect("couldn't convert a slice into a Vector"))
            }
        }

        impl<T> Default for Vector<T, $d>
        where
            T: Number<Output = T>
                + Copy
        {
            #[inline]
            fn default() -> Self {
                T::zero().into()
            }
        }

        macro_rules! impl_op {
            ($trait:ident, $fname:ident, $op:tt) => {
                impl<T> std::ops::$trait for Vector<T, $d>
                where
                    T: std::ops::$trait<Output = T>
                        + Copy
                {
                    type Output = Self;

                    #[inline]
                    #[must_use]
                    fn $fname(self, rhs: Self) -> Self {
                        Self([
                            $(self.0[$index] $op rhs.0[$index]),+
                        ])
                    }
                }

                impl<'a, 'b, T> std::ops::$trait<&'b Vector<T, $d>> for &'a Vector<T, $d>
                where
                    T: std::ops::$trait<Output = T>
                        + Copy
                {
                    type Output = Vector<T, $d>;

                    #[inline]
                    #[must_use]
                    fn $fname(self, rhs: &'b Vector<T, $d>) -> Self::Output {
                        Vector::<T, $d>([
                            $(self.0[$index] $op rhs.0[$index]),+
                        ])
                    }
                }

                impl<T> std::ops::$trait<T> for Vector<T, $d>
                where
                    T: std::ops::$trait<Output = T>
                        + Copy
                {
                    type Output = Self;

                    #[inline]
                    #[must_use]
                    fn $fname(self, rhs: T) -> Self {
                        Self([
                            $(self.0[$index] $op rhs),+
                        ])
                    }
                }
            }
        }

        macro_rules! impl_op_assign {
            ($trait:ident, $fname:ident, $op:tt) => {
                impl<T> std::ops::$trait for Vector<T, $d>
                where
                    T: std::ops::$trait
                        + Copy
                {
                    #[inline]
                    fn $fname(&mut self, rhs: Self) {
                        $(self.0[$index] $op rhs.0[$index]);+
                    }
                }

                impl<'a, T> std::ops::$trait<&'a Vector<T, $d>> for Vector<T, $d>
                where
                    T: std::ops::$trait
                        + Copy
                {
                    #[inline]
                    fn $fname(&mut self, rhs: &'a Vector<T, $d>) {
                        $(self.0[$index] $op rhs.0[$index]);+
                    }
                }

                impl<T> std::ops::$trait<T> for Vector<T, $d>
                where
                    T: std::ops::$trait
                        + Copy
                {
                    #[inline]
                    fn $fname(&mut self, rhs: T) {
                        $(self.0[$index] $op rhs);+
                    }
                }
            }
        }

        impl_op!(Add, add, +);
        impl_op!(Sub, sub, -);
        impl_op!(Mul, mul, *);
        impl_op!(Div, div, /);
        impl_op_assign!(AddAssign, add_assign, +=);
        impl_op_assign!(SubAssign, sub_assign, -=);
        impl_op_assign!(MulAssign, mul_assign, *=);
        impl_op_assign!(DivAssign, div_assign, /=);
    }
}
