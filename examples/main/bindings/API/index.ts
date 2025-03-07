// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { AuthenticateParams } from "./Params";
import type { EmptyParams } from "../Nitram";
import type { GetTokenParams } from "./Params";
import type { IdParams } from "../Nitram";
import type { SendMessageParams } from "./Params";
import type { User } from "../User";

export type AuthenticateAPI = { i: AuthenticateParams, o: boolean, };

export type GetTokenAPI = { i: GetTokenParams, o: string, };

export type GetUserAPI = { i: IdParams, o: User, };

export type MessagesAPI = { i: EmptyParams, o: Array<string>, };

export type SendMessageAPI = { i: SendMessageParams, o: Array<string>, };
