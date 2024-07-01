function first_plan_price(plans){
    return plans.get().get(0).item.get("price_id").get();
}