//! One straight sRGB float representation for native paint and browser CSS.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedColor { components:[f32;4], hex:Option<[u8;3]> }
impl ResolvedColor {
    pub const fn rgb8(r:u8,g:u8,b:u8)->Self {Self{components:[r as f32/255.,g as f32/255.,b as f32/255.,1.],hex:Some([r,g,b])}}
    pub const fn transparent()->Self {Self{components:[0.;4],hex:Some([0;3])}}
    pub fn components(self)->[f32;4] {self.components}
    pub fn alpha(self)->f32 {self.components[3]}
    pub(crate) fn opacity(mut self,value:f32)->Self {self.components[3]*=value;self}
    pub fn css(self)->String {
        if self.alpha()==1. && let Some([r,g,b])=self.hex {return format!("#{r:02x}{g:02x}{b:02x}");}
        let [r,g,b,a]=self.components;format!("color(srgb {r} {g} {b} / {a})")
    }
    pub(super) fn parse(text:&str)->Result<Self,String> {
        let text=text.trim();
        if text.starts_with('#') && matches!(text.len(),7|9) && text[1..].bytes().all(|v|v.is_ascii_hexdigit()) {
            let byte=|index|u8::from_str_radix(&text[index..index+2],16).expect("checked hex bytes");
            let mut value=Self::rgb8(byte(1),byte(3),byte(5));if text.len()==9{value.components[3]=byte(7) as f32/255.;}return Ok(value);
        }
        if let Some(body)=text.strip_prefix("oklch(").and_then(|text|text.strip_suffix(')')) {
            let spaced=body.replace('/'," / ");let parts=spaced.split_ascii_whitespace().collect::<Vec<_>>();
            if parts.len()!=3 && !(parts.len()==5 && parts[3]=="/"){return Err("oklch() requires lightness, chroma, hue and optional / alpha".into());}
            let scalar=|value:&str,percent:bool|->Result<f64,String>{
                let (value,scale)=if let Some(value)=value.strip_suffix('%'){if !percent{return Err("OKLCH chroma and hue require unitless numbers".into());}(value,0.01)}else{(value,1.)};
                let value=value.parse::<f64>().map_err(|_|"OKLCH components must be finite numbers".to_owned())?*scale;
                if !value.is_finite(){return Err("OKLCH components must be finite numbers".into());}Ok(value)
            };
            let l=scalar(parts[0],true)?;let c=scalar(parts[1],false)?;let h=scalar(parts[2],false)?;
            let a=if parts.len()==5{scalar(parts[4],true)?}else{1.};
            if !(0. ..=1.).contains(&l) || c<0. || !(0. ..=360.).contains(&h) || !(0. ..=1.).contains(&a) {
                return Err("OKLCH lightness/alpha must be 0–1 or 0–100%; chroma nonnegative; hue 0–360 degrees".into());
            }
            let parsed=color::parse_color(text).map_err(|e|format!("Invalid OKLCH color: {e}"))?.to_alpha_color::<color::Srgb>();
            let components=parsed.components;
            // Keep transform roundoff rather than clipping it or quantizing to bytes.
            if components[..3].iter().any(|v|!v.is_finite() || *v < -0.000001 || *v > 1.000001) {
                return Err("Color is outside the supported sRGB gamut; choose an in-gamut color".into());
            }
            return Ok(Self{components,hex:None});
        }
        Err("Color must be #RRGGBB, #RRGGBBAA or an in-gamut oklch() value, or a @token alias".into())
    }
}
