// Theme management script
(function() {
    'use strict';
    
    const THEME_ATTRIBUTE = 'data-bs-theme';
    const HTML_ELEMENT = document.documentElement;
    
    /**
     * Get the system theme preference
     * @returns {string} 'dark' or 'light'
     */
    function getSystemTheme() {
        if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
            return 'dark';
        }
        return 'light';
    }
    
    /**
     * Apply the theme to the document
     * @param {string} theme - 'dark' or 'light'
     */
    function applyTheme(theme) {
        HTML_ELEMENT.setAttribute(THEME_ATTRIBUTE, theme);
        
        // Update charts if chartHelper is available
        if (window.chartHelper && typeof window.chartHelper.updateAllCharts === 'function') {
            // Use setTimeout to ensure DOM is updated
            setTimeout(() => {
                window.chartHelper.updateAllCharts();
            }, 0);
        }
        
        // Dispatch custom event for theme change
        window.dispatchEvent(new CustomEvent('themechange', { detail: { theme } }));
    }
    
    /**
     * Initialize theme on page load
     */
    function initTheme() {
        const theme = getSystemTheme();
        applyTheme(theme);
    }
    
    /**
     * Listen for system theme changes
     */
    function watchSystemTheme() {
        if (window.matchMedia) {
            const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
            
            // Always listen for system theme changes
            mediaQuery.addEventListener('change', (e) => {
                const newTheme = e.matches ? 'dark' : 'light';
                applyTheme(newTheme);
            });
        }
    }
    
    // Initialize theme immediately to prevent flash of wrong theme
    initTheme();
    
    // Watch for system theme changes
    watchSystemTheme();
    
    // Expose theme functions to window for potential Blazor interop
    window.themeManager = {
        getTheme: getSystemTheme,
        setTheme: applyTheme,
        getSystemTheme: getSystemTheme
    };
})();

