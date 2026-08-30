//! Ports `Trainer/Components/DecimalAmountInput.razor` and the
//! `decimal-input.js` shim it drove.
//!
//! # Calculator-style entry
//!
//! The field shows a formatted string but the model is a raw integer: typed
//! digits flow in from the right and the decimal point is positional only, so
//! typing `1`, `2`, `5` at two places walks `0.01` -> `0.12` -> `1.25`. Every
//! keystroke re-derives the text from the digits, which is why the field has to
//! be rewritten on input rather than left to accept what was typed.
//!
//! # Why this writes the element instead of binding its value
//!
//! A controlled `value:` binding only rewrites the DOM when the *rendered*
//! string changes. Typing a non-digit does not change the value — `1a` and `1`
//! both extract to `1` — so the stray character would stay on screen. The shim
//! assigned `el.value` unconditionally for exactly this reason, so this does
//! too, and moves the caret to the end afterwards as the shim did.
//!
//! Everything else the shim carried is gone: no `DotNetObjectReference`, no
//! `attach`/`detach` pair, no `_lastValue`/`_lastPlaces` bookkeeping to suppress
//! re-syncs, and no `JSDisconnectedException` handling during teardown.

use dioxus::prelude::*;
use trainer_core::helpers::amount;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

/// The formatted text for a value, empty when the field is cleared.
///
/// **Narrowing from the shim.** `decimalInput._format` took `Math.abs`, so a
/// negative value displayed unsigned; `DecimalAmount.Format`, which the shim's
/// own comment names as the reference, keeps the sign. This follows the C#.
/// Digits typed into the field are never negative, so the two agree on
/// everything reachable through the UI.
fn text_for(value: Option<i32>, decimal_places: i32) -> String {
    value.map_or_else(String::new, |v| amount::format(v, decimal_places))
}

/// Writes the formatted text and parks the caret at the end.
fn sync(element: &HtmlInputElement, text: &str) {
    element.set_value(text);
    // JS string offsets are UTF-16 units; the text is ASCII digits and a dot,
    // but counting units rather than bytes keeps that an observation, not an
    // assumption.
    let end = text.encode_utf16().count() as u32;
    // The shim wrapped this in try/catch: not every input type supports
    // selection, and a field that has never been focused can reject it.
    let _ = element.set_selection_range(end, end);
}

#[component]
pub fn DecimalAmountInput(
    value: Option<i32>,
    /// Digits after the point. 0 behaves as a plain integer field.
    decimal_places: i32,
    on_change: EventHandler<Option<i32>>,
    #[props(default = "form-control".to_owned())] class: String,
    #[props(default = false)] disabled: bool,
) -> Element {
    let mut element = use_signal(|| None::<HtmlInputElement>);

    // Ports `decimalInput.sync`: the value or the precision changing from
    // outside — a different activity type selected, an activity loaded for
    // edit — has to reach a field this component does not otherwise control.
    // Props are not signals, so the dependency has to be declared explicitly.
    use_effect(use_reactive!(|(value, decimal_places)| {
        let text = text_for(value, decimal_places);
        if let Some(input) = element.read().as_ref()
            && input.value() != text
        {
            sync(input, &text);
        }
    }));

    rsx! {
        input {
            r#type: "text",
            inputmode: "numeric",
            class: "{class}",
            // Shown when the field is empty so the precision stays visible.
            placeholder: amount::format(0, decimal_places),
            disabled,
            onmounted: move |event| {
                let input = event
                    .downcast::<web_sys::Element>()
                    .and_then(|el| el.clone().dyn_into::<HtmlInputElement>().ok());
                if let Some(input) = input {
                    // Ports `decimalInput.attach`'s initial render. Doing it here
                    // rather than leaving it to the effect above removes any
                    // dependence on whether effects run before or after mount.
                    input.set_value(&text_for(value, decimal_places));
                    element.set(Some(input));
                }
            },
            oninput: move |event| {
                let digits = amount::extract_digits(Some(&event.value()));
                if let Some(input) = element.read().as_ref() {
                    sync(input, &text_for(digits, decimal_places));
                }
                on_change.call(digits);
            },
        }
    }
}
