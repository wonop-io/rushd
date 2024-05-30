use yew::prelude::*;
use yew_router::prelude::*;
use gloo::net::http::Request;
use crate::routes::Route;
use api_types::{
  ExampleApiType, ApiResponse
};
use serde_json::from_str;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let state = use_state(|| ExampleApiType::default());

    {
        let state = state.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let response_text = Request::get("/api/hello-world")
                    .send()
                    .await
                    .unwrap()
                    .text()
                    .await
                    .unwrap();
                let api_response: ApiResponse<ExampleApiType> = from_str(&response_text).unwrap();
                if let Some(data) = api_response.data {
                    state.set(data);
                }
            });
            || ()
        });
    }

    html! {
        <div class="text-white flex flex-col items-center justify-center h-screen">
          <span class="text-2xl">{"Data fetched: "}{state.payload.clone()}</span>
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
            </div>
        </BrowserRouter>
    }
}
