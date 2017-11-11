use arena::Arena;
use json::JsonValue;
use number::Number;

pub trait IntoJson<'arena> {
    fn into_json(self, arena: &'arena Arena) -> &'arena JsonValue<'arena>;
}

impl<'a, 'arena> IntoJson<'arena> for &'a str {
    #[inline]
    fn into_json(self, arena: &'arena Arena) -> &'arena JsonValue<'arena> {
        let string = arena.alloc_str(self);

        arena.alloc(JsonValue::String(string))
    }
}

impl<'arena, T: Into<Number>> IntoJson<'arena> for T {
    fn into_json(self, arena: &'arena Arena) -> &'arena JsonValue<'arena> {
        let number = self.into();

        arena.alloc(JsonValue::Number(number))
    }
}

impl<'arena> IntoJson<'arena> for bool {
    fn into_json(self, arena: &'arena Arena) -> &'arena JsonValue<'arena> {
        arena.alloc(JsonValue::Boolean(self))
    }
}
