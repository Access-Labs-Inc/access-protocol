import { ErrorMessage } from "./errors";
import { NonceResponse, LoginResponse } from "./routes";

type Result = ErrorMessage | NonceResponse | LoginResponse;

export class ApiResponse {
  success: boolean;
  result: Result | undefined;
  constructor(success: boolean = false, result?: Result) {
    this.result = result;
    this.success = success;
  }
}
