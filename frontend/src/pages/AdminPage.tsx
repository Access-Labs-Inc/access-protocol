import React from "react";
import AdmiMintCard from "../components/AdminMintCard";
import CentralStateCard from "../components/CentralStateCard";
import AllPoolsCard from "../components/AllPoolsCard";
import EditInflation from "../components/EditInflation";

const AdminPage = () => {
  return (
    <div
      style={{
        height: "100%",
        width: "100%",
        display: "flex",
        alignItems: "space-around",
        justifyContent: "space-around",
        flexDirection: "column",
      }}
    >
      <AdmiMintCard />
      <EditInflation />
      <CentralStateCard />
      <AllPoolsCard />
    </div>
  );
};

export default AdminPage;
