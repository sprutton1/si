import { User, Workspace } from "@prisma/client";
import { JwtPayload } from "jsonwebtoken";
import * as Koa from "koa";
import { nanoid } from "nanoid";
import _ from "lodash";
import { CustomAppContext, CustomAppState } from "../custom-state";
import { ApiError } from "../lib/api-error";
import { setCache } from "../lib/cache";
import { createJWT, verifyJWT } from "../lib/jwt";
import { tryCatch } from "../lib/try-catch";
import { getUserById, UserId } from "./users.service";
import { getWorkspaceById, WorkspaceId } from "./workspaces.service";
import { posthog } from "../lib/posthog";

export const SI_COOKIE_NAME = "si-auth";

export type AuthProviders = "google" | "github" | "password";

// TODO: figure out the shape of the JWT and what data we want

// Auth tokens used for communication between the user's browser and this auth api
type AuthTokenData = {
  userId: string;
  workspaceId?: string;
};

// will figure out what we want to pass in here...
export function createAuthToken(userId: string) {
  const payload: AuthTokenData = {
    userId,
  };
  return createJWT(payload);
}

export async function decodeAuthToken(token: string) {
  const verified = verifyJWT(token);
  // if token was an sdf token, it will be scoped to a workspace and use its terminology (pk)
  if (typeof verified !== "string" && "user_pk" in verified) {
    return {
      userId: verified.user_pk,
      workspaceId: verified.workspace_pk,
      ..._.omit(verified, ["user_pk", "workspace_pk"]),
    } as AuthTokenData & JwtPayload;
  } else {
    return verified as AuthTokenData & JwtPayload;
  }
}

// Auth tokens used for communication between the user's browser and SDF
// and between that SDF instance and this auth api if necessary
export type SdfAuthTokenPayload = SdfAuthTokenPayloadV1 | SdfAuthTokenPayloadV2;
export type CurrentSdfAuthTokenPayload = SdfAuthTokenPayloadV2;
export interface SdfAuthTokenPermission {
  roles: ["web" | "automation"];
}

interface SdfAuthTokenPayloadV2 {
  version: 2;
  userId: UserId;
  workspaceId: WorkspaceId;
  allow: SdfAuthTokenPermission[];
}

// Old auth token versions
interface SdfAuthTokenPayloadV1 {
  user_pk: string;
  workspace_pk: string;
}

// will figure out what we want to pass in here...
export function createSdfAuthToken(payload: SdfAuthTokenPayloadV2) {
  // can add more metadata, expiry, etc...
  return createJWT(payload, { subject: payload.userId });
}

export async function decodeSdfAuthToken(token: string) {
  return verifyJWT(token) as SdfAuthTokenPayload & JwtPayload;
}

function wipeAuthCookie(ctx: Koa.Context) {
  ctx.cookies.set(SI_COOKIE_NAME, null);
}

export const loadAuthMiddleware: Koa.Middleware<CustomAppState, CustomAppContext> = async (ctx, next) => {
  let authToken = ctx.cookies.get(SI_COOKIE_NAME);
  if (!authToken && ctx.headers.authorization) {
    authToken = ctx.headers.authorization.split(" ").pop();
  }
  if (!authToken) {
    // special auth handling only used in tests
    if (process.env.NODE_ENV === "test" && ctx.headers["spoof-auth"]) {
      const user = await getUserById(ctx.headers["spoof-auth"] as string);
      if (!user) throw new Error("spoof auth user does not exist");
      ctx.state.authUser = user;
    }

    return next();
  }

  const decoded = await tryCatch(() => {
    return decodeAuthToken(authToken!);
  }, (_err) => {
    // TODO: check the type of error before handling this way

    // clear the cookie and return an error
    wipeAuthCookie(ctx);
    throw new ApiError("Unauthorized", "AuthTokenCorrupt", "Invalid auth token");
  });

  // console.log(decoded);

  // make sure cookie is valid - not sure if this can happen...
  if (!decoded) {
    wipeAuthCookie(ctx);
    throw new ApiError("Unauthorized", "AuthTokenCorrupt", "Invalid auth token");
  }
  // TODO: deal with various other errors, logout on all devices, etc...

  const user = await getUserById(decoded.userId);

  if (!user) {
    wipeAuthCookie(ctx);
    throw new ApiError("Unauthorized", "AuthUserMissing", "Cannot find user data");
  }

  ctx.state.authUser = user;

  if (decoded.workspaceId) {
    const workspace = await getWorkspaceById(decoded.workspaceId);
    if (!workspace) {
      wipeAuthCookie(ctx);
      throw new ApiError("Unauthorized", "AuthWorkspaceMissing", "Cannot find workspace data");
    }
    ctx.state.authWorkspace = workspace;
  }

  return next();
};

export async function beginAuthConnect(workspace: Workspace, user: User) {
  // generate a new single use authentication code that we will send to the instance
  const connectCode = nanoid(24);
  await setCache(`auth:connect:${connectCode}`, {
    workspaceId: workspace.id,
    userId: user.id,
  }, { expiresIn: 60 });

  return await makeAuthConnectUrl(workspace, user, connectCode);
}

export async function makeAuthConnectUrl(workspace: Workspace, user: User, code: string, redirect?: string) {
  const params: { [key: string]: string } = { code };

  const onDemandAssets = await posthog.isFeatureEnabled("on_demand_assets", user.id);
  if (onDemandAssets) {
    params.onDemandAssets = `true`;
  }
  if (redirect) {
    params.redirect = redirect;
  }

  const paramsString = Object.keys(params).map((key) => `${key}=${params[key]}`).join("&");

  return `${workspace.instanceUrl}/auth-connect?${paramsString}`;
}
