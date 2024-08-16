// Create a new script element
var script = document.createElement('script');
script.async = true;
script.src = "https://www.googletagmanager.com/gtag/js?id=G-1KLPVDHHMG";

// Append the script to the head or body
document.head.appendChild(script);

// Once the script is loaded, configure Google Analytics
script.onload = function() {
    window.dataLayer = window.dataLayer || [];
    function gtag(){dataLayer.push(arguments);}
    gtag('js', new Date());
    gtag('config', 'G-1KLPVDHHMG');
};
