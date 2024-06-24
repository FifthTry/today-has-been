

class PaymentForm extends HTMLElement {
    constructor() {
        super();
        this.attachShadow({ mode: 'open' });
        this.shadowRoot.innerHTML = `
<!--            <style>-->
<!--                /* Add your styles here */-->
<!--            </style>-->
<!--            <form id="payment-form">-->
<!--                <div id="payment-element"></div>-->
<!--                <button type="submit" id="submit" style="background-color: #00AEC4; color: white;">Subscribe</button>-->
<!--                <div id="error-message"></div>-->
<!--            </form>-->
        `;
    }

    connectedCallback() {
        this.loadStripe().then(() => this.initializeStripe());
    }

    async initializeStripe() {
        let data = window.ftd.component_data(this);
        let client_secret = data.payment.get().get("client_secret").get();
        let stripe_key = data.payment.get().get("stripe_key").get();
        let return_url = data.payment.get().get("return_url").get();

        const stripe = Stripe(stripe_key);
        const options = {
            clientSecret: client_secret,
            appearance: {/*...*/},
        };
        const elements = stripe.elements(options);
        const paymentElement = elements.create('payment');
        // paymentElement.mount(this.shadowRoot.querySelector('#payment-element'));
        // const form = this.shadowRoot.querySelector('#payment-form');
        paymentElement.mount('#payment-element');
        const form = document.getElementById('payment-form');
        form.addEventListener('submit', async (event) => {
            event.preventDefault();
            this.shadowRoot.querySelector("#submit").style.display = 'none';
            const priceId = this.shadowRoot.querySelector('input[name="price_id"]:checked').value;

            const { error } = await stripe.confirmSetup({
                elements,
                confirmParams: {
                    return_url: `${return_url}&price_id=${priceId}`,
                }
            });

            if (error) {
                const messageContainer = this.shadowRoot.querySelector('#error-message');
                messageContainer.textContent = error.message;
            }
        });
    }

    loadStripe() {
        return new Promise((resolve, reject) => {
            const script = document.createElement('script');
            script.src = 'https://js.stripe.com/v3/';
            script.onload = resolve;
            script.onerror = reject;
            document.head.appendChild(script);

            document.getElementById('payment-form')
                .replaceWith(this.createForm());
        });
    }

    createForm() {
        const form = document.createElement('form');
        form.id = 'payment-form';

        const paymentElementDiv = document.createElement('div');
        paymentElementDiv.id = 'payment-element';

        const submitButton = document.createElement('button');
        submitButton.type = 'submit';
        submitButton.id = 'submit';
        submitButton.textContent = 'Submit';

        const errorMessageDiv = document.createElement('div');
        errorMessageDiv.id = 'error-message';

        form.appendChild(paymentElementDiv);
        form.appendChild(submitButton);
        form.appendChild(errorMessageDiv);

        return form;
    }

}

customElements.define('show-payment', PaymentForm);