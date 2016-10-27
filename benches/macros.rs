// macro_rules! count {
//     () => {
//         0
//     };
//     ($first:ident $(, $rest:ident)*) => {
//         1 + count!($($rest),*)
//     };
// }

// // c_enum!(HttpProtocol {
// //     HTTP_PROTOCOL_UNKNOWN,
// //     HTTP10,
// //     HTTP11,
// // });
// #[macro_export]
// macro_rules! c_enum {
//     ($name:ident { $($variant:ident,)* }) => {
//         #[allow(non_camel_case_types)]
//         #[derive(Copy, Clone, Debug, PartialEq)]
//         enum $name {
//             $(
//                 $variant,
//             )*
//         }

//         impl ::rustc_serialize::Encodable for $name {
//             #[inline]
//             fn encode<S>(&self, s: &mut S) -> Result<(), S::Error>
//                 where S: ::rustc_serialize::Encoder,
//             {
//                 (*self as usize).encode(s)
//             }
//         }

//         impl ::rustc_serialize::Decodable for $name {
//             #[inline]
//             fn decode<D>(d: &mut D) -> Result<Self, D::Error>
//                 where D: ::rustc_serialize::Decoder,
//             {
//                 match ::num_traits::FromPrimitive::from_usize(try!(d.read_usize())) {
//                     Some(value) => Ok(value),
//                     None => Err(d.error("cannot convert from usize")),
//                 }
//             }
//         }

//         impl ::num_traits::FromPrimitive for $name {
//             #[inline]
//             fn from_i64(i: i64) -> Option<Self> {
//                 ::num_traits::FromPrimitive::from_u64(i as u64)
//             }

//             #[inline]
//             fn from_u64(n: u64) -> Option<Self> {
//                 static VARIANTS: [$name; count!($($variant),*)] = [$($name::$variant),*];
//                 VARIANTS.get(n as usize).map(|v| *v)
//             }
//         }

//         impl ::serde::Serialize for $name {
//             #[inline]
//             fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
//                 where S: ::serde::Serializer,
//             {
//                 serializer.serialize_u64(*self as u64)
//             }
//         }

//         impl ::serde::Deserialize for $name {
//             #[inline]
//             fn deserialize<D>(de: &mut D) -> Result<Self, D::Error>
//                 where D: ::serde::Deserializer
//             {
//                 Ok(::num_traits::FromPrimitive::from_u64(
//                     try!(u64::deserialize(de))
//                 ).unwrap())
//             }
//         }
//     };
// }
