function first_plan_price(plans){
    return plans.get().get(0).item.get("price_id").get();
}


function setTimezone() {
    const timeZone = Intl.DateTimeFormat().resolvedOptions().timeZone;
    ftd.http("/api/v0.1/user/timezone/", "POST", null, { timezone: timeZone });
    return 1;
}


// Code taken from https://stackoverflow.com/questions/1091372/getting-the-clients-time-zone-and-offset-in-javascript

function setTimezoneOffset() {
    var offset = new Date().getTimezoneOffset(), o = Math.abs(offset);
    const timeZone = (offset < 0 ? "+" : "-") + ("00" + Math.floor(o / 60)).slice(-2) + ":" + ("00" + (o % 60)).slice(-2);
    ftd.http("/api/v0.1/user/timezone/", "POST", null, { timezone: timeZone });
    return 1;
}

