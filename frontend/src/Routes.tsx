import React from "react";
import { HashRouter, Route, Routes } from "react-router-dom";
import HomePage from "./pages/HomePage";
import CreatePool from "./pages/CreatePool";
import NavigationFrame from "./components/NavigationFrame";
import Stake from "./pages/Stake";
import CentralState from "./pages/CentralState";
import AllPools from "./pages/AllPools";
import InspectPool from "./pages/InspectPool";
import AdminMint from "./pages/AdminMint";
import JWT from "./pages/JWT";

export default function RoutesApp() {
  return (
    <HashRouter>
      <NavigationFrame>
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/jwt" element={<JWT />} />
          <Route path="/stake" element={<Stake />} />
          <Route path="/create-pool" element={<CreatePool />} />
          <Route path="/central-state" element={<CentralState />} />
          <Route path="/all-pools" element={<AllPools />} />
          <Route path="/pool/:key" element={<InspectPool />} />
          <Route path="/admin-mint" element={<AdminMint />} />
        </Routes>
      </NavigationFrame>
    </HashRouter>
  );
}
