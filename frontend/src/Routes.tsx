import React, { useEffect } from "react";
import { HashRouter, Route, Routes } from "react-router-dom";
import HomePage from "./pages/HomePage";

export default function RoutesApp() {
  return (
    <HashRouter>
      <Routes>
        <Route path="/" element={<HomePage />} />
      </Routes>
    </HashRouter>
  );
}
