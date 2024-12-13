use crate::database::animation::Animation;

#[derive(Debug)]
pub enum ChangeLighting {
    // this is only to write things, not for reads
    Brightness(u8),
    Animation(Animation),
    Speed(f64),
}
