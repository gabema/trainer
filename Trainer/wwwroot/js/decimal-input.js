// Calculator-style decimal entry. The field holds a formatted string but the model
// is a raw integer accumulator: typed digits flow in from the right and the decimal
// point is positional only. Mirrors Trainer/Helpers/DecimalAmount.cs — keep in sync.
window.decimalInput = {
    _maxDigits: 9,

    // Raw integer -> formatted string for a given precision (null -> empty).
    _format: function (value, places) {
        if (value === null || value === undefined || value === '') {
            return '';
        }
        let s = String(Math.abs(parseInt(value, 10)));
        if (places <= 0) {
            return s;
        }
        while (s.length < places + 1) {
            s = '0' + s;
        }
        return s.slice(0, s.length - places) + '.' + s.slice(s.length - places);
    },

    // Arbitrary field text -> raw integer (digits only, capped), or null when empty.
    _digits: function (text) {
        let d = (String(text).match(/\d/g) || []).join('');
        if (d.length > this._maxDigits) {
            d = d.slice(0, this._maxDigits);
        }
        return d.length === 0 ? null : parseInt(d, 10);
    },

    attach: function (el, dotnetRef, places, value) {
        el.dataset.places = places;
        el.value = this._format(value, places);
        const self = this;
        const handler = function () {
            const p = parseInt(el.dataset.places || '0', 10);
            const n = self._digits(el.value);
            el.value = self._format(n, p);
            const end = el.value.length;
            try { el.setSelectionRange(end, end); } catch (e) { /* type may not support selection */ }
            dotnetRef.invokeMethodAsync('OnInput', n);
        };
        el._decimalHandler = handler;
        el.addEventListener('input', handler);
    },

    // Re-render the field when the bound value or precision changes from outside
    // (e.g. selecting a different activity type, or loading an activity to edit).
    sync: function (el, places, value) {
        el.dataset.places = places;
        el.value = this._format(value, places);
    },

    detach: function (el) {
        if (el && el._decimalHandler) {
            el.removeEventListener('input', el._decimalHandler);
            delete el._decimalHandler;
        }
    }
};
