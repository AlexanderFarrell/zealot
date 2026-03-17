import { AttributeAPI } from "./attribute";
import { AuthAPI } from "./auth";
import { BaseAPI } from "./common";
import { AttributeKindAPI } from './attribute_kind';
import { ItemTypeAPI } from "./item_type";
import { CommentAPI } from "./comment";
import { RepeatAPI } from "./repeat";
import { RuleAPI } from "./rule";

class ZealotAPI extends BaseAPI {
    public Auth: AuthAPI;
    public Attribute: AttributeAPI;
    public AttributeKind: AttributeKindAPI;
    public ItemType: ItemTypeAPI;
    public Comment: CommentAPI;
    public Repeat: RepeatAPI;
    public Rule: RuleAPI;

    public constructor(baseUrl: string) {
        super(baseUrl)
        this.Auth = new AuthAPI(baseUrl);
        this.Attribute = new AttributeAPI(baseUrl);
        this.AttributeKind = new AttributeKindAPI(baseUrl);
        this.Comment = new CommentAPI(baseUrl);
        this.ItemType = new ItemTypeAPI(baseUrl);
        this.Repeat = new RepeatAPI(baseUrl);
        this.Rule = new RuleAPI(baseUrl);
    }
}

export default ZealotAPI;