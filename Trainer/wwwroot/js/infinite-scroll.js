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

