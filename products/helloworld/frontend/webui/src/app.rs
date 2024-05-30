use crate::pages::Login;
use crate::pages::RequestAccess;
use crate::routes::Route;
use yew::prelude::*;
use yew_router::components::Link;
use yew_router::prelude::*;
use yewdux::prelude::*;
use gloo::net::http::Request;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let state = use_state(|| String::new());

    {
        let state = state.clone();
        use_effect_with_deps(move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let fetched_data = Request::get("/api/hello-world")
                    .send()
                    .await
                    .unwrap()
                    .text()
                    .await
                    .unwrap();
                state.set(fetched_data);
            });
            || ()
        }, ());
    }

    html! {
        <div class="bg-gray-900 min-h-screen">
            <Switch<Route> render={render} />
            <div>{"Data fetched: "}{(*state).clone()}</div>
        </div>
    }
}


#[function_component(App)]
pub fn app() -> Html {

  let render = move |routes| match routes {
    Route::HomePage => {
        html! {<HomePage />}
    }
};

  
    html! {
        <BrowserRouter>
            <div class="bg-gray-900 min-h-screen">
                <Switch<Route> render={render} />
                <div>{"Data fetched: "}{(*state).clone()}</div>
            </div>
        </BrowserRouter>
    }
}
