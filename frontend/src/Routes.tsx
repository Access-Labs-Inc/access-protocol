import React, { useEffect } from "react";
import { HashRouter, Route, Routes } from "react-router-dom";
import HomePage from "./pages/HomePage";
import CreatePool from "./pages/CreatePool";
import NavigationFrame from "./components/NavigationFrame";
import Stake from "./pages/Stake";
import AllPools from "./components/AllPoolsCard";
import InspectPool from "./pages/InspectPool";
import JWT from "./pages/JWT";
import AdminPage from "./pages/AdminPage";
import { useSnackbar } from "notistack";
import BondPage from "./pages/BondPage";
import SnackbarUtils from "./utils/SnackbarUtils";

export default function RoutesApp() {
  const { enqueueSnackbar, closeSnackbar } = useSnackbar();
  useEffect(() => {
    SnackbarUtils.setSnackBar(enqueueSnackbar, closeSnackbar);
  }, [enqueueSnackbar, closeSnackbar]);
  return (
    <HashRouter>
      <NavigationFrame>
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/jwt" element={<JWT />} />
          <Route path="/stake" element={<Stake />} />
          <Route path="/create-pool" element={<CreatePool />} />
          <Route path="/all-pools" element={<AllPools />} />
          <Route path="/pool/:key" element={<InspectPool />} />
          <Route path="/bond/:key" element={<BondPage />} />
          <Route path="/admin" element={<AdminPage />} />
        </Routes>
      </NavigationFrame>
    </HashRouter>
  );
}
