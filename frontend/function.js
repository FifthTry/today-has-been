function first_plan_price(plans){
    return plans.get().get(0).item.get("price_id").get();
}

// Not calling this function in the frontend.
function setTimezone() {
    const timeZone = Intl.DateTimeFormat().resolvedOptions().timeZone;
    ftd.http("/api/v0.1/user/timezone/", "POST", null, { timezone: timeZone });
    return 1;
}


// Code taken from https://stackoverflow.com/questions/1091372/getting-the-clients-time-zone-and-offset-in-javascript
function setTimezoneOffset() {
    var offset = new Date().getTimezoneOffset(), o = Math.abs(offset);
    // Fetch fastn-sid from browser cookie
    let sid = getCookieValue("fastn-sid")
    const timeZone = (offset < 0 ? "+" : "-") + ("00" + Math.floor(o / 60)).slice(-2) + ":" + ("00" + (o % 60)).slice(-2);
    ftd.http("/api/v0.1/user/timezone/", "POST", null, { timezone: timeZone, sid: sid });
    return 1;
}

function redirectToFreePlan(access_token) {
    var offset = new Date().getTimezoneOffset(), o = Math.abs(offset);
    // Fetch fastn-sid from browser cookie
    const timeZone = (offset < 0 ? "+" : "-") + ("00" + Math.floor(o / 60)).slice(-2) + ":" + ("00" + (o % 60)).slice(-2);
    ftd.http("/api/v0.1/user/timezone/", "POST", null, { timezone: timeZone, sid: access_token });
    window.location.href = `/get-free-plan?authtoken=${access_token}`;
    return 1;
}


function getCookieValue(name) {
    const cookies = document.cookie.split(';');
    for (let cookie of cookies) {
        cookie = cookie.trim();
        if (cookie.startsWith(name + '=')) {
            return cookie.substring(name.length + 1);
        }
    }
    return null;
}


