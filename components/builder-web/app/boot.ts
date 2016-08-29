// Copyright (c) 2016 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

///<reference path="../typings/browser/index.d.ts" />
///<reference path='../node_modules/immutable/dist/immutable.d.ts'/>

// Include the nav control from the main website
require("./zepto-custom");
import "./nav";

import "reflect-metadata";
require("zone.js/dist/zone");
import { bind, enableProdMode } from "@angular/core";
import { bootstrap } from "@angular/platform-browser-dynamic";
import { LocationStrategy, HashLocationStrategy } from "@angular/common";
import { APP_ROUTER_PROVIDER } from "./routes";
import { AppComponent } from "./AppComponent";
import { AppStore } from "./AppStore";
import config from "./config";

// This mess can be taken out once we're live.
let goingToBoot = true;

if (config["environment"] === "production") {
    enableProdMode();

    // Don't load if we're on habitat.sh and the "friends" cookie is not set
    if (config["friends_only"] &&
        window.location.host.endsWith("habitat.sh") &&
        !document.cookie.includes("habitat_is_not_bldr")) {
        goingToBoot = false;
    }
}

if (goingToBoot) {
    bootstrap(AppComponent, [
        AppStore,
        APP_ROUTER_PROVIDER,
        // Temporarily adding this until we have nginx handle routing non-existent
        // pages.
        bind(LocationStrategy).toClass(HashLocationStrategy)
    ]);
}
