import { Router } from "express";
import { validateToken } from "../utils/jwt";
import path from "path";

const router = Router();

/**
 * Example of a route protected by JWT
 */
router.get("/", validateToken, (req, res) => {
  res.sendStatus(200);
});

export default router;
