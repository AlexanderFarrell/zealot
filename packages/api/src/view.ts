import { get_json, post_json } from "@websoil/engine";
import { ViewResult } from "@zealot/domain/src/saved_view";
import type {
    SavedViewDefinition,
    ViewInspectDto,
    ViewResultDto,
} from "@zealot/domain/src/saved_view";
import { BaseAPI } from "./common";

/** Why a view call failed: a bad definition (400) carries the server's message. */
export class ViewError extends Error {
    public constructor(
        public readonly kind: "invalid" | "not_found" | "other",
        message: string,
        /** The item a failed embed pointed at, when it exists but is not a Saved View. */
        public readonly itemId?: number,
    ) {
        super(message);
        this.name = "ViewError";
    }
}

async function translate<T>(call: () => Promise<T>): Promise<T> {
    try {
        return await call();
    } catch (error) {
        const response = (error as Error & { response?: Response }).response;
        if (!response) throw error;
        if (response.status === 404) throw new ViewError("not_found", "View not found");
        if (response.status === 400) {
            const text = await response.clone().text().catch(() => "");
            throw new ViewError("invalid", text || "Invalid view definition");
        }
        throw new ViewError("other", (error as Error).message);
    }
}

export interface ViewPage {
    limit?: number;
    offset?: number;
}

export class ViewAPI extends BaseAPI {
    /** Run a saved view (`POST /view/{id}/run`). */
    public async Run(viewId: number, page: ViewPage = {}): Promise<ViewResult> {
        const dto = await translate(
            () => post_json(`${this.baseUrl}/view/${viewId}/run`, page) as Promise<ViewResultDto>,
        );
        return ViewResult.fromDto(dto);
    }

    /** Run an unsaved definition, for the builder preview (`POST /view/run`). */
    public async RunDefinition(definition: unknown, page: ViewPage = {}): Promise<ViewResult> {
        const dto = await translate(
            () => post_json(`${this.baseUrl}/view/run`, { definition, ...page }) as Promise<ViewResultDto>,
        );
        return ViewResult.fromDto(dto);
    }

    /** How a saved view resolves: the query after token expansion, today, cost. */
    public async Inspect(viewId: number): Promise<ViewInspectDto> {
        return translate(() => get_json(`${this.baseUrl}/view/${viewId}/inspect`) as Promise<ViewInspectDto>);
    }

    /** Validate an unsaved definition without running it for results. */
    public async Validate(definition: SavedViewDefinition | unknown): Promise<ViewInspectDto> {
        return translate(
            () => post_json(`${this.baseUrl}/view/validate`, { definition }) as Promise<ViewInspectDto>,
        );
    }
}
