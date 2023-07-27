use crate::Color;

use glam::Vec3;
use kolor::details::color::{RGBPrimaries, WhitePoint};

pub trait ColorEncoding: Sized + 'static {
    /// The raw data representation used by this encoding.
    type Repr: ColorRepr;

    /// The 'bag of components' this color encoding uses.
    type ComponentStruct: ComponentStructFor<Self::Repr>;

    /// The [`LinearColorSpace`] used by this encoding.
    type LinearSpace: LinearColorSpace;

    /// Used in `Debug` and `Default` implementations.
    const NAME: &'static str;
    
    /// Convert from `Self::Repr` to a `glam::Vec3` in the `Self::LinearSpace` color space and a separate
    /// (not pre-multiplied) alpha component. If this encoding does not have alpha, return 1.0.
    fn src_transform_raw(repr: Self::Repr) -> (Vec3, f32);

    /// Convert from a `glam::Vec3` in `Self::LinearSpace` and separate alpha to a `Self::Repr`. If this encoding
    /// does not have alpha, you can disregard it.
    fn dst_transform_raw(raw: Vec3, alpha: f32) -> Self::Repr;
}

/// Implementing this trait for a struct marks that it is safe to pointer cast `Repr` as `Self`.
pub unsafe trait ComponentStructFor<Repr: ColorRepr>: Sized + Clone + Copy + 'static {
    fn cast(repr: &Repr) -> &Self;
    fn cast_mut(repr: &mut Repr) -> &mut Self;
}

/// Implemented by the raw data representation of a color encoding
pub trait ColorRepr: Sized + Clone + Copy + 'static {
    /// The type of a single element of this repr
    type Element: Sized + Clone + Copy + 'static;
}

/// Implemented by color encodings that can do alpha compositing
pub trait AlphaComposite: ColorEncoding {
    fn composite(over: Self::Repr, under: Self::Repr) -> Self::Repr;
}

/// Implemented by color encodings that can perform saturate-style clamping.
pub trait Saturate: ColorEncoding {
    fn saturate(repr: Self::Repr) -> Self::Repr;
}

/// Implemented by color encodings which can blend from one color to another based on a blending factor.
/// 
/// It is expected that this blending function should be implemented as similar to a linear interpolation,
/// and should be fairly cheap.
pub trait Blend: ColorEncoding {
    fn blend(from: Self::Repr, to: Self::Repr, factor: f32) -> Self::Repr;
}

/// Marks a type as representing a color encoding in which it makes sense to be able to perform mathematical
/// operations on the contained color values directly.
pub trait WorkingEncoding: ColorEncoding {}

/// A type that implements [`LinearColorSpace`] represents a color space which can be defined by a *linear transformation only*
/// (i.e. a 3x3 matrix multiplication) from the CIE XYZ color space.
/// 
/// A linear color space is defined by the combination of a set of [Primaries][RGBPrimaries] and a [White Point][WhitePoint].
pub trait LinearColorSpace {
    const PRIMARIES: RGBPrimaries;
    const WHITE_POINT: WhitePoint;
}

/// A trait that marks `Self` as being a color encoding which is able to be directly converted from `SrcEnc`,
/// as well as allowing some hooks to perform extra mapping during the conversion if necessary.
pub trait ConvertFrom<SrcEnc>
where
    SrcEnc: ColorEncoding,
    Self: ColorEncoding,
    Self::LinearSpace: LinearConvertFromRaw<SrcEnc::LinearSpace>,
{
    /// If required or desired, perform a mapping of some kind to the input
    /// before it undergoes its source transform. This may be desirable to perform some form of
    /// gamut mapping if the src encoding has a larger size of representable colors than te dst encoding.
    #[inline(always)]
    fn map_src(_src: &mut SrcEnc::Repr) { }
}

/// Performs the raw conversion from the [`LinearColorSpace`] represented by `SrcSpc` to
/// the [`LinearColorSpace`] represented by `Self`.
pub trait LinearConvertFromRaw<SrcSpace: LinearColorSpace>: LinearColorSpace {
    fn linear_part_raw(raw: &mut Vec3);
}

pub trait ColorInto<DstCol> {
    fn color_into(self) -> DstCol;
}

impl<SrcEnc, DstEnc> ColorInto<Color<DstEnc>> for Color<SrcEnc>
where
    SrcEnc: ColorEncoding,
    DstEnc: ColorEncoding + ConvertFrom<SrcEnc>,
    DstEnc::LinearSpace: LinearConvertFromRaw<SrcEnc::LinearSpace>,
{
    #[inline(always)]
    fn color_into(self) -> Color<DstEnc> {
        self.convert()
    }
}

// /// A "conversion query" for a [`Color`][crate::Color].
// ///
// /// A type that implements this
// /// trait is able to be used as the type parameter for [`Color::convert`][crate::Color::convert].
// ///
// /// The types that implement this trait are:
// /// * [`ColorSpace`] types
// /// * [`Color`][crate::Color] types (in which case it will be converted to that color's space)
// pub trait ColorConversionQuery<SrcSpace: LinearColorSpace, St: State> {
//     type DstSpace: ConvertFromRaw<SrcSpace>;
// }

// impl<SrcSpace, DstSpace, St> ColorConversionQuery<SrcSpace, St> for Color<DstSpace, St>
// where
//     SrcSpace: LinearColorSpace,
//     DstSpace: ConvertFromRaw<SrcSpace>,
//     St: State,
// {
//     type DstSpace = DstSpace;
// }
