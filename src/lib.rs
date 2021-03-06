//! General-purpose sum types.
//!
//! <p style="font-family: 'Fira Sans',sans-serif;padding:0.3em 0"><strong>
//! <a href="https://crates.io/crates/sum">📦&nbsp;&nbsp;Crates.io</a>&nbsp;&nbsp;│&nbsp;&nbsp;<a href="https://github.com/alecmocatta/sum">📑&nbsp;&nbsp;GitHub</a>&nbsp;&nbsp;│&nbsp;&nbsp;<a href="https://constellation.zulipchat.com/#narrow/stream/213236-subprojects">💬&nbsp;&nbsp;Chat</a>
//! </strong></p>
//!
//! Arbitrarily-sized product types exist in Rust in the form of [tuples](https://doc.rust-lang.org/std/primitive.tuple.html). This is a generalisation of bluss's [Either](https://docs.rs/either/1.5.0/either/enum.Either.html) type to provide **arbitrarily-sized sum types**\*.
//!
//! \* Over up to 32 types.

#![doc(html_root_url = "https://docs.rs/sum/0.1.7")]
#![warn(
	missing_copy_implementations,
	missing_debug_implementations,
	// missing_docs,
	trivial_casts,
	trivial_numeric_casts,
	unused_import_braces,
	unused_qualifications,
	unused_results,
	clippy::pedantic
)] // from https://github.com/rust-unofficial/patterns/blob/master/anti_patterns/deny-warnings.md
#![allow(unused_variables, unreachable_patterns)]
#![allow(
	clippy::empty_enum,
	clippy::type_complexity,
	clippy::wrong_self_convention,
	clippy::must_use_candidate,
	clippy::unsafe_derive_deserialize,
	clippy::match_wildcard_for_single_variants
)]

#[cfg(feature = "futures")]
use futures_core::stream::Stream;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{
	error::Error, fmt::{Display, Formatter, Result}, future::Future, hint::unreachable_unchecked, ops::{Deref, DerefMut}, pin::Pin, task::{Context, Poll}
};

macro_rules! impl_sum {
	(@into_inner $name:ident : $($t:ident)* : !) => (
		impl $name {
			pub fn into_inner(self) -> ! {
				match self { }
			}
		}
	);
	(@into_inner $name:ident : $($t:ident)* : $first_a:ident $($a:ident)*) => (
		impl<$first_a> $name<$first_a, $($a,)*> {
			pub fn into_inner(self) -> $first_a {
				match self {
					$($name::$t(inner) => inner,)*
				}
			}
		}
	);
	($name:ident : $($t:ident $is:ident $map:ident $get:ident)* : $first_a:tt $($a:ident)* ) => (
		#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
		#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
		pub enum $name<$($t,)*> {
			$($t($t),)*
		}
		impl<$($t,)*> $name<$($t,)*> {
			$(
			pub fn $is(&self) -> bool {
				match *self {
					$name::$t(_) => true,
					_ => false
				}
			}
			// TODO
			// pub fn $map<F,O>(self, f: F) -> $name<$(&$t,)*> where F: FnOnce($t)->O {
			// 	match self {
			// 		$name::$t(a) => $name::$t(f(a)),
			// 		$($name::$t(inner) => inner,)*
			// 	}
			// }
			pub fn $get(self) -> Option<$t> {
				match self {
					$name::$t(a) => Some(a),
					_ => None
				}
			}
			)*
			pub fn as_ref(&self) -> $name<$(&$t,)*> {
				match *self {
					$($name::$t(ref a) => $name::$t(a),)*
				}
			}
			pub fn as_mut(&mut self) -> $name<$(&mut $t,)*> {
				match *self {
					$($name::$t(ref mut a) => $name::$t(a),)*
				}
			}
			pub fn as_pin_ref<'a>(self: Pin<&'a Self>) -> $name<$(Pin<&'a $t>,)*> {
				unsafe {
					match self.get_ref() {
						$($name::$t(a) => $name::$t(Pin::new_unchecked(a)),)*
						_ => unreachable_unchecked(),
					}
				}
			}
			pub fn as_pin_mut<'a>(self: Pin<&'a mut Self>) -> $name<$(Pin<&'a mut $t>,)*> {
				unsafe {
					match self.get_unchecked_mut() {
						$($name::$t(a) => $name::$t(Pin::new_unchecked(a)),)*
						_ => unreachable_unchecked(),
					}
				}
			}
		}
		impl_sum!(@into_inner $name : $($t)* : $first_a $($a)*);
		impl<$($t,)* Target> AsRef<Target> for $name<$($t,)*>
		where
			$($t: AsRef<Target>,)*
		{
			fn as_ref(&self) -> &Target {
				match *self {
					$($name::$t(ref inner) => inner.as_ref(),)*
				}
			}
		}
		impl<$($t,)* Target> AsMut<Target> for $name<$($t,)*>
		where
			$($t: AsMut<Target>,)*
		{
			fn as_mut(&mut self) -> &mut Target {
				match *self {
					$($name::$t(ref mut inner) => inner.as_mut(),)*
				}
			}
		}
		impl<$($t,)*> Error for $name<$($t,)*>
		where
			$($t: Error,)*
		{
			#[allow(deprecated)]
			fn description(&self) -> &str {
				match *self {
					$($name::$t(ref inner) => inner.description(),)*
				}
			}
			#[allow(deprecated)]
			fn cause(&self) -> Option<&dyn Error> {
				match *self {
					$($name::$t(ref inner) => inner.cause(),)*
				}
			}
			fn source(&self) -> Option<&(dyn Error + 'static)> {
				match *self {
					$($name::$t(ref inner) => inner.source(),)*
				}
			}
		}
		impl<$($t,)*> Display for $name<$($t,)*>
		where
			$($t: Display,)*
		{
			fn fmt(&self, f: &mut Formatter) -> Result {
				match *self {
					$($name::$t(ref inner) => inner.fmt(f),)*
				}
			}
		}
		impl_sum!(@multi $name : $($t $is $map $get)* : $first_a $($a)*);
	);
	(@multi $name:ident : : $first_a:tt $($a:ident)* ) => ();
	(@multi $name:ident : $first_t:ident $first_is:ident $first_map:ident $first_get:ident $($t:ident $is:ident $map:ident $get:ident)* : $first_a:tt $($a:ident)* ) => (
		impl<$first_t, $($t,)*> Deref for $name<$first_t, $($t,)*>
		where
			$first_t: Deref,
			$($t: Deref<Target = $first_t::Target>,)*
		{
			type Target = $first_t::Target;
			fn deref(&self) -> &Self::Target {
				match *self {
					$name::$first_t(ref inner) => &*inner,
					$($name::$t(ref inner) => &*inner,)*
				}
			}
		}
		impl<$first_t, $($t,)*> DerefMut for $name<$first_t, $($t,)*>
		where
			$first_t: DerefMut,
			$($t: DerefMut<Target = $first_t::Target>,)*
		{
			fn deref_mut(&mut self) -> &mut Self::Target {
				match *self {
					$name::$first_t(ref mut inner) => &mut *inner,
					$($name::$t(ref mut inner) => &mut *inner,)*
				}
			}
		}

		impl<$first_t, $($t,)*> Iterator for $name<$first_t, $($t,)*>
		where
			$first_t: Iterator,
			$($t: Iterator<Item = $first_t::Item>,)*
		{
			type Item = <$first_t>::Item;

			fn next(&mut self) -> Option<Self::Item> {
				match *self {
					$name::$first_t(ref mut inner) => inner.next(),
					$($name::$t(ref mut inner) => inner.next(),)*
				}
			}
			fn size_hint(&self) -> (usize, Option<usize>) {
				match *self {
					$name::$first_t(ref inner) => inner.size_hint(),
					$($name::$t(ref inner) => inner.size_hint(),)*
				}
			}
		}
		impl<$first_t, $($t,)*> DoubleEndedIterator for $name<$first_t, $($t,)*>
		where
			$first_t: DoubleEndedIterator,
			$($t: DoubleEndedIterator<Item = $first_t::Item>,)*
		{
			fn next_back(&mut self) -> Option<Self::Item> {
				match *self {
					$name::$first_t(ref mut inner) => inner.next_back(),
					$($name::$t(ref mut inner) => inner.next_back(),)*
				}
			}
		}
		impl<$first_t, $($t,)*> ExactSizeIterator for $name<$first_t, $($t,)*>
		where
			$first_t: ExactSizeIterator,
			$($t: ExactSizeIterator<Item = $first_t::Item>,)*
		{
			fn len(&self) -> usize {
				match *self {
					$name::$first_t(ref inner) => inner.len(),
					$($name::$t(ref inner) => inner.len(),)*
				}
			}
		}

		impl<$first_t, $($t,)*> Future for $name<$first_t, $($t,)*>
		where
			$first_t: Future,
			$($t: Future<Output = $first_t::Output>,)*
		{
			type Output = <$first_t>::Output;

			fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
				match self.as_pin_mut() {
					$name::$first_t(inner) => inner.poll(cx),
					$($name::$t(inner) => inner.poll(cx),)*
				}
			}
		}
		#[cfg(feature = "futures")]
		impl<$first_t, $($t,)*> Stream for $name<$first_t, $($t,)*>
		where
			$first_t: Stream,
			$($t: Stream<Item = $first_t::Item>,)*
		{
			type Item = <$first_t>::Item;

			fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
				match self.as_pin_mut() {
					$name::$first_t(inner) => inner.poll_next(cx),
					$($name::$t(inner) => inner.poll_next(cx),)*
				}
			}
		}
	);
}

// TODO: impl_sum!(A Sum1 B Sum2 C Sum3 D Sum4 E Sum5 F Sum6);
#[cfg(feature = "0")]
impl_sum!(Sum0: : !);
#[cfg(feature = "1")]
impl_sum!(Sum1: A is_a map_a a: A);
#[cfg(feature = "2")]
impl_sum!(Sum2: A is_a map_a a B is_b map_b b: A A);
#[cfg(feature = "3")]
impl_sum!(Sum3: A is_a map_a a B is_b map_b b C is_c map_c c: A A A);
#[cfg(feature = "4")]
impl_sum!(Sum4: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d: A A A A);
#[cfg(feature = "5")]
impl_sum!(Sum5: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e: A A A A A);
#[cfg(feature = "6")]
impl_sum!(Sum6: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f: A A A A A A);
#[cfg(feature = "7")]
impl_sum!(Sum7: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g: A A A A A A A);
#[cfg(feature = "8")]
impl_sum!(Sum8: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h: A A A A A A A A);
#[cfg(feature = "9")]
impl_sum!(Sum9: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i: A A A A A A A A A);
#[cfg(feature = "10")]
impl_sum!(Sum10: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i J is_j map_j j: A A A A A A A A A A);
#[cfg(feature = "11")]
impl_sum!(Sum11: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i J is_j map_j j K is_k map_k k: A A A A A A A A A A A);
#[cfg(feature = "12")]
impl_sum!(Sum12: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i J is_j map_j j K is_k map_k k L is_l map_l l: A A A A A A A A A A A A);
#[cfg(feature = "13")]
impl_sum!(Sum13: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i J is_j map_j j K is_k map_k k L is_l map_l l M is_m map_m m: A A A A A A A A A A A A A);
#[cfg(feature = "14")]
impl_sum!(Sum14: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i J is_j map_j j K is_k map_k k L is_l map_l l M is_m map_m m N is_n map_n n: A A A A A A A A A A A A A A);
#[cfg(feature = "15")]
impl_sum!(Sum15: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i J is_j map_j j K is_k map_k k L is_l map_l l M is_m map_m m N is_n map_n n O is_o map_o o: A A A A A A A A A A A A A A A);
#[cfg(feature = "16")]
impl_sum!(Sum16: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i J is_j map_j j K is_k map_k k L is_l map_l l M is_m map_m m N is_n map_n n O is_o map_o o P is_p map_p p: A A A A A A A A A A A A A A A A);
#[cfg(feature = "17")]
impl_sum!(Sum17: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i J is_j map_j j K is_k map_k k L is_l map_l l M is_m map_m m N is_n map_n n O is_o map_o o P is_p map_p p Q is_q map_q q: A A A A A A A A A A A A A A A A A);
#[cfg(feature = "18")]
impl_sum!(Sum18: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i J is_j map_j j K is_k map_k k L is_l map_l l M is_m map_m m N is_n map_n n O is_o map_o o P is_p map_p p Q is_q map_q q R is_r map_r r: A A A A A A A A A A A A A A A A A A);
#[cfg(feature = "19")]
impl_sum!(Sum19: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i J is_j map_j j K is_k map_k k L is_l map_l l M is_m map_m m N is_n map_n n O is_o map_o o P is_p map_p p Q is_q map_q q R is_r map_r r S is_s map_s s: A A A A A A A A A A A A A A A A A A A);
#[cfg(feature = "20")]
impl_sum!(Sum20: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i J is_j map_j j K is_k map_k k L is_l map_l l M is_m map_m m N is_n map_n n O is_o map_o o P is_p map_p p Q is_q map_q q R is_r map_r r S is_s map_s s T is_t map_t t: A A A A A A A A A A A A A A A A A A A A);
#[cfg(feature = "21")]
impl_sum!(Sum21: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i J is_j map_j j K is_k map_k k L is_l map_l l M is_m map_m m N is_n map_n n O is_o map_o o P is_p map_p p Q is_q map_q q R is_r map_r r S is_s map_s s T is_t map_t t U is_u map_u u: A A A A A A A A A A A A A A A A A A A A A);
#[cfg(feature = "22")]
impl_sum!(Sum22: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i J is_j map_j j K is_k map_k k L is_l map_l l M is_m map_m m N is_n map_n n O is_o map_o o P is_p map_p p Q is_q map_q q R is_r map_r r S is_s map_s s T is_t map_t t U is_u map_u u V is_v map_v v: A A A A A A A A A A A A A A A A A A A A A A);
#[cfg(feature = "23")]
impl_sum!(Sum23: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i J is_j map_j j K is_k map_k k L is_l map_l l M is_m map_m m N is_n map_n n O is_o map_o o P is_p map_p p Q is_q map_q q R is_r map_r r S is_s map_s s T is_t map_t t U is_u map_u u V is_v map_v v W is_w map_w w: A A A A A A A A A A A A A A A A A A A A A A A);
#[cfg(feature = "24")]
impl_sum!(Sum24: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i J is_j map_j j K is_k map_k k L is_l map_l l M is_m map_m m N is_n map_n n O is_o map_o o P is_p map_p p Q is_q map_q q R is_r map_r r S is_s map_s s T is_t map_t t U is_u map_u u V is_v map_v v W is_w map_w w X is_x map_x x: A A A A A A A A A A A A A A A A A A A A A A A A);
#[cfg(feature = "25")]
impl_sum!(Sum25: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i J is_j map_j j K is_k map_k k L is_l map_l l M is_m map_m m N is_n map_n n O is_o map_o o P is_p map_p p Q is_q map_q q R is_r map_r r S is_s map_s s T is_t map_t t U is_u map_u u V is_v map_v v W is_w map_w w X is_x map_x x Y is_y map_y y: A A A A A A A A A A A A A A A A A A A A A A A A A);
#[cfg(feature = "26")]
impl_sum!(Sum26: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i J is_j map_j j K is_k map_k k L is_l map_l l M is_m map_m m N is_n map_n n O is_o map_o o P is_p map_p p Q is_q map_q q R is_r map_r r S is_s map_s s T is_t map_t t U is_u map_u u V is_v map_v v W is_w map_w w X is_x map_x x Y is_y map_y y Z is_z map_z z: A A A A A A A A A A A A A A A A A A A A A A A A A A);
#[cfg(feature = "27")]
impl_sum!(Sum27: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i J is_j map_j j K is_k map_k k L is_l map_l l M is_m map_m m N is_n map_n n O is_o map_o o P is_p map_p p Q is_q map_q q R is_r map_r r S is_s map_s s T is_t map_t t U is_u map_u u V is_v map_v v W is_w map_w w X is_x map_x x Y is_y map_y y Z is_z map_z z Aa is_aa map_aa aa: A A A A A A A A A A A A A A A A A A A A A A A A A A A);
#[cfg(feature = "28")]
impl_sum!(Sum28: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i J is_j map_j j K is_k map_k k L is_l map_l l M is_m map_m m N is_n map_n n O is_o map_o o P is_p map_p p Q is_q map_q q R is_r map_r r S is_s map_s s T is_t map_t t U is_u map_u u V is_v map_v v W is_w map_w w X is_x map_x x Y is_y map_y y Z is_z map_z z Aa is_aa map_aa aa Ab is_ab map_ab ab: A A A A A A A A A A A A A A A A A A A A A A A A A A A A);
#[cfg(feature = "29")]
impl_sum!(Sum29: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i J is_j map_j j K is_k map_k k L is_l map_l l M is_m map_m m N is_n map_n n O is_o map_o o P is_p map_p p Q is_q map_q q R is_r map_r r S is_s map_s s T is_t map_t t U is_u map_u u V is_v map_v v W is_w map_w w X is_x map_x x Y is_y map_y y Z is_z map_z z Aa is_aa map_aa aa Ab is_ab map_ab ab Ac is_ac map_ac ac: A A A A A A A A A A A A A A A A A A A A A A A A A A A A A);
#[cfg(feature = "30")]
impl_sum!(Sum30: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i J is_j map_j j K is_k map_k k L is_l map_l l M is_m map_m m N is_n map_n n O is_o map_o o P is_p map_p p Q is_q map_q q R is_r map_r r S is_s map_s s T is_t map_t t U is_u map_u u V is_v map_v v W is_w map_w w X is_x map_x x Y is_y map_y y Z is_z map_z z Aa is_aa map_aa aa Ab is_ab map_ab ab Ac is_ac map_ac ac Ad is_ad map_ad ad: A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A);
#[cfg(feature = "31")]
impl_sum!(Sum31: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i J is_j map_j j K is_k map_k k L is_l map_l l M is_m map_m m N is_n map_n n O is_o map_o o P is_p map_p p Q is_q map_q q R is_r map_r r S is_s map_s s T is_t map_t t U is_u map_u u V is_v map_v v W is_w map_w w X is_x map_x x Y is_y map_y y Z is_z map_z z Aa is_aa map_aa aa Ab is_ab map_ab ab Ac is_ac map_ac ac Ad is_ad map_ad ad Ae is_ae map_ae ae: A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A);
#[cfg(feature = "32")]
impl_sum!(Sum32: A is_a map_a a B is_b map_b b C is_c map_c c D is_d map_d d E is_e map_e e F is_f map_f f G is_g map_g g H is_h map_h h I is_i map_i i J is_j map_j j K is_k map_k k L is_l map_l l M is_m map_m m N is_n map_n n O is_o map_o o P is_p map_p p Q is_q map_q q R is_r map_r r S is_s map_s s T is_t map_t t U is_u map_u u V is_v map_v v W is_w map_w w X is_x map_x x Y is_y map_y y Z is_z map_z z Aa is_aa map_aa aa Ab is_ab map_ab ab Ac is_ac map_ac ac Ad is_ad map_ad ad Ae is_ae map_ae ae Af is_af map_af af: A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A A);

#[doc(hidden)]
#[macro_export]
macro_rules! sum2 {
    (impl<$($t:ident),*> $trait:ident <$($p:ty),*> for Sum where {
        $(type $assoc:ident;)*
        $(mut fn $mut_fn:ident(&mut self $(, $mut_arg:ident : $mut_arg_ty:ty)*) -> $mut_ret:ty;)*
        $(fn $ref_fn:ident(&self $(, $ref_arg:ident : $ref_arg_ty:ty)*) -> $ref_ret:ty;)*
    }) => (
        impl<A, B, $($t,)*> $trait<$($t,)*> for $crate::Sum2<A, B>
        where
            A: $trait<$($t,)*>,
            B: $trait<$($t,)* $($assoc = A::$assoc,)*>,
        {
            $(type $assoc = A::$assoc;)*

            $(
            #[inline]
            fn $mut_fn(&mut self $(, $mut_arg : $mut_arg_ty)*) -> $mut_ret {
                match self {
                    $crate::Sum2::A(self_) => self_.$mut_fn($($mut_arg),*),
                    $crate::Sum2::B(self_) => self_.$mut_fn($($mut_arg),*),
                }
            }
            )*

            $(
            #[inline]
            fn $ref_fn(&self $(, $ref_arg : $ref_arg_ty)*) -> $ref_ret {
                match self {
                    $crate::Sum2::A(self_) => self_.$ref_fn($($ref_arg),*),
                    $crate::Sum2::B(self_) => self_.$ref_fn($($ref_arg),*),
                }
            }
            )*
        }
    )
}
#[doc(hidden)]
#[macro_export]
macro_rules! sum3 {
    (impl<$($t:ident),*> $trait:ident <$($p:ty),*> for Sum where {
        $(type $assoc:ident;)*
        $(mut fn $mut_fn:ident(&mut self $(, $mut_arg:ident : $mut_arg_ty:ty)*) -> $mut_ret:ty;)*
        $(fn $ref_fn:ident(&self $(, $ref_arg:ident : $ref_arg_ty:ty)*) -> $ref_ret:ty;)*
    }) => (
        impl<A, B, C, $($t,)*> $trait<$($t,)*> for $crate::Sum3<A, B, C>
        where
            A: $trait<$($t,)*>,
            B: $trait<$($t,)* $($assoc = A::$assoc,)*>,
            C: $trait<$($t,)* $($assoc = A::$assoc,)*>,
        {
            $(type $assoc = A::$assoc;)*

            $(
            #[inline]
            fn $mut_fn(&mut self $(, $mut_arg : $mut_arg_ty)*) -> $mut_ret {
                match self {
                    $crate::Sum3::A(self_) => self_.$mut_fn($($mut_arg),*),
                    $crate::Sum3::B(self_) => self_.$mut_fn($($mut_arg),*),
                    $crate::Sum3::C(self_) => self_.$mut_fn($($mut_arg),*),
                }
            }
            )*

            $(
            #[inline]
            fn $ref_fn(&self $(, $ref_arg : $ref_arg_ty)*) -> $ref_ret {
                match self {
                    $crate::Sum3::A(self_) => self_.$ref_fn($($ref_arg),*),
                    $crate::Sum3::B(self_) => self_.$ref_fn($($ref_arg),*),
                    $crate::Sum3::C(self_) => self_.$ref_fn($($ref_arg),*),
                }
            }
            )*
        }
    )
}

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! derive_sum {
    (impl<$($t:ident),*> $trait:ident <$($p:ty),*> for Sum where {
        $(type $assoc:ident;)*
        $(mut fn $mut_fn:ident(&mut self $(, $mut_arg:ident : $mut_arg_ty:ty)*) -> $mut_ret:ty;)*
        $(fn $ref_fn:ident(&self $(, $ref_arg:ident : $ref_arg_ty:ty)*) -> $ref_ret:ty;)*
    }) => (
        sum2!(
            impl<$($t),*> $trait <$($p),*> for Sum where {
                $(type $assoc;)*
                $(mut fn $mut_fn(&mut self $(, $mut_arg : $mut_arg_ty)*) -> $mut_ret;)*
                $(fn $ref_fn(&self $(, $ref_arg : $ref_arg_ty)*) -> $ref_ret;)*
            }
        );
        sum3!(
            impl<$($t),*> $trait <$($p),*> for Sum where {
                $(type $assoc;)*
                $(mut fn $mut_fn(&mut self $(, $mut_arg : $mut_arg_ty)*) -> $mut_ret;)*
                $(fn $ref_fn(&self $(, $ref_arg : $ref_arg_ty)*) -> $ref_ret;)*
            }
        );
    );
    (impl $trait:ident for Sum {
        $(type $assoc:ident;)*
        $(mut fn $mut_fn:ident(&mut self $(, $mut_arg:ident : $mut_arg_ty:ty)*) -> $mut_ret:ty;)*
        $(fn $ref_fn:ident(&self $(, $ref_arg:ident : $ref_arg_ty:ty)*) -> $ref_ret:ty;)*
    }) => (
        sum2!(
            impl<> $trait <> for Sum where {
                $(type $assoc;)*
                $(mut fn $mut_fn(&mut self $(, $mut_arg : $mut_arg_ty)*) -> $mut_ret;)*
                $(fn $ref_fn(&self $(, $ref_arg : $ref_arg_ty)*) -> $ref_ret;)*
            }
        );
        sum3!(
            impl<> $trait <> for Sum where {
                $(type $assoc;)*
                $(mut fn $mut_fn(&mut self $(, $mut_arg : $mut_arg_ty)*) -> $mut_ret;)*
                $(fn $ref_fn(&self $(, $ref_arg : $ref_arg_ty)*) -> $ref_ret;)*
            }
        );
    );
}

#[test]
fn basic() {
	let mut e = Sum2::A(2);
	let r = Sum2::B(2);
	assert_eq!(e, Sum2::A(2));
	e = r;
	assert_eq!(e, Sum2::B(2));
	assert_eq!(e.a(), None);
	assert_eq!(e.b(), Some(2));
	assert_eq!(e.as_ref().b(), Some(&2));
	assert_eq!(e.as_mut().b(), Some(&mut 2));
}

#[test]
fn derive() {
	trait Abc {
		fn abc(&self);
		fn def(&mut self);
	}
	derive_sum!(impl Abc for Sum {
		mut fn def(&mut self) -> ();
		fn abc(&self) -> ();
	});
}
