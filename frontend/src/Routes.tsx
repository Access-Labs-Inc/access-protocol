import React, { useEffect } from "react";
import { HashRouter, Route, Routes } from "react-router-dom";
import HomePage from "./pages/HomePage";
import CreatePool from "./pages/CreatePool";
import NavigationFrame from "./components/NavigationFrame";

export default function RoutesApp() {
  return (
    <HashRouter>
      <NavigationFrame>
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/jwt" element={<HomePage />} />
          <Route path="/stake" element={<HomePage />} />
          <Route path="/create-pool" element={<CreatePool />} />
        </Routes>
      </NavigationFrame>
    </HashRouter>
  );
}
