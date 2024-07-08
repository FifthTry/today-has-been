function first_plan_price(plans){
    return plans.get().get(0).item.get("price_id").get();
}


function callTimezone() {
    const timeZone = Intl.DateTimeFormat().resolvedOptions().timeZone;
    ftd.http("/api/v0.1/user/timezone/", "POST", null, { timezone: timeZone });
    return 1;
}

