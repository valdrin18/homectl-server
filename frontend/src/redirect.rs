use dioxus::prelude::*;
use dioxus_router::RouterContext;

#[derive(PartialEq, Props)]
pub struct RedirectProps<'a> {
    to: &'a str,
}

#[allow(non_snake_case)]
pub fn Redirect<'a>(cx: Scope<'a, RedirectProps<'a>>) -> Element<'a> {
    if let Some(service) = cx.consume_context::<RouterContext>() {
        service.push_route(cx.props.to, None, None)
    }

    None
}
