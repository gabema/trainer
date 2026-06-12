let observer = null;
let dotNetRef = null;
let currentElementId = null;

export function initializeObserver(dotNetReference, elementId) {
    dotNetRef = dotNetReference;
    currentElementId = elementId;
    
    if (observer) {
        observer.disconnect();
    }
    
    observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                if (dotNetRef) {
                    dotNetRef.invokeMethodAsync('OnScrollTriggerVisible');
                }
            }
        });
    }, {
        root: null,
        rootMargin: '0px',
        threshold: 0.1
    });
    
    observeElement(elementId);
}

export function observeElement(elementId) {
    if (!observer) {
        return;
    }

    const element = document.getElementById(elementId);
    if (element) {
        // Re-observe: calling observe() on an already-observed target is a no-op, so when the
        // trigger stays within the viewport (sparse filtered results) the callback would never
        // re-fire. Unobserving first forces a fresh intersection evaluation. (issue #85)
        observer.unobserve(element);
        observer.observe(element);
    }
}

export function dispose() {
    if (observer) {
        observer.disconnect();
        observer = null;
    }
    dotNetRef = null;
    currentElementId = null;
}

