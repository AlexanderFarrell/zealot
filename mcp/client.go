package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"mime/multipart"
	"net/http"
	"net/url"
	"os"
	"path/filepath"
	"strings"
	"time"
)

// Item mirrors the Zealot Item struct from item_data.go.
type Item struct {
	ItemID     int            `json:"item_id"`
	Title      string         `json:"title"`
	Content    string         `json:"content"`
	Attributes map[string]any `json:"attributes"`
	Types      []ItemType     `json:"types"`
}

type ItemType struct {
	TypeID                int      `json:"type_id"`
	Name                  string   `json:"name"`
	Description           string   `json:"description"`
	IsSystem              bool     `json:"is_system"`
	RequiredAttributeKeys []string `json:"required_attribute_keys"`
}

type AttributeFilter struct {
	Key      string `json:"key"`
	Op       string `json:"op"`
	Value    any    `json:"value"`
	ListMode string `json:"list_mode"`
}

type AccountDetails struct {
	Username  string          `json:"username"`
	Email     string          `json:"email"`
	Name      string          `json:"name"`
	AccountID int             `json:"account_id"`
	Settings  json.RawMessage `json:"settings"`
}

type APIKeyStatus struct {
	Exists bool `json:"exists"`
}

type AttributeKind struct {
	KindID      int             `json:"kind_id"`
	Key         string          `json:"key"`
	Description string          `json:"description"`
	BaseType    string          `json:"base_type"`
	Config      json.RawMessage `json:"config"`
	IsSystem    bool            `json:"is_system"`
}

type CommentEntry struct {
	CommentID int64     `json:"comment_id"`
	Item      Item      `json:"item"`
	Timestamp time.Time `json:"timestamp"`
	Content   string    `json:"content"`
}

type TrackerEntry struct {
	TrackerID int64     `json:"tracker_id"`
	Item      Item      `json:"item"`
	Timestamp time.Time `json:"timestamp"`
	Level     int       `json:"level"`
	Comment   string    `json:"comment"`
}

type ScoreEntry struct {
	Score     int       `json:"score"`
	Timestamp time.Time `json:"timestamp"`
}

type RepeatStatusDate struct {
	Status  string    `json:"status"`
	Item    Item      `json:"item"`
	Date    time.Time `json:"date"`
	Comment string    `json:"comment"`
}

type FileStat struct {
	Path     string `json:"path"`
	Size     int64  `json:"size"`
	IsFolder bool   `json:"is_folder"`
	Modified int64  `json:"modified_at"`
}

type MediaDirectory struct {
	Files []FileStat `json:"files"`
}

type CreateItemInput struct {
	Title      string
	Content    string
	Attributes map[string]any
	TypeNames  []string
}

type CreateItemTypeInput struct {
	Name                  string
	Description           string
	RequiredAttributeKeys []string
}

type ZealotClient struct {
	baseURL    string
	token      string
	httpClient *http.Client
}

func NewZealotClient(baseURL, token string) *ZealotClient {
	return &ZealotClient{
		baseURL: strings.TrimRight(baseURL, "/"),
		token:   token,
		httpClient: &http.Client{
			Timeout: 20 * time.Second,
		},
	}
}

func (c *ZealotClient) apiURL(path string) string {
	return c.baseURL + path
}

func (c *ZealotClient) do(req *http.Request, out any) error {
	req.Header.Set("Authorization", "Bearer "+c.token)
	if req.Header.Get("Accept") == "" {
		req.Header.Set("Accept", "application/json")
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return err
	}

	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		message := strings.TrimSpace(string(body))
		if message == "" {
			message = http.StatusText(resp.StatusCode)
		}
		return fmt.Errorf("HTTP %d: %s", resp.StatusCode, message)
	}

	if out == nil || len(bytes.TrimSpace(body)) == 0 {
		return nil
	}

	if err := json.Unmarshal(body, out); err != nil {
		return fmt.Errorf("decoding %s %s response: %w", req.Method, req.URL.Path, err)
	}
	return nil
}

func (c *ZealotClient) request(method, path string, payload any, out any) error {
	var body io.Reader
	if payload != nil {
		raw, err := json.Marshal(payload)
		if err != nil {
			return err
		}
		body = bytes.NewReader(raw)
	}

	req, err := http.NewRequest(method, c.apiURL(path), body)
	if err != nil {
		return err
	}
	if payload != nil {
		req.Header.Set("Content-Type", "application/json")
	}

	return c.do(req, out)
}

func escapePathSegments(path string) string {
	trimmed := strings.Trim(path, "/")
	if trimmed == "" {
		return ""
	}

	parts := strings.Split(trimmed, "/")
	for i, part := range parts {
		parts[i] = url.PathEscape(part)
	}
	return strings.Join(parts, "/")
}

func (c *ZealotClient) mediaPath(path string) string {
	escaped := escapePathSegments(path)
	if escaped == "" {
		return "/api/media/"
	}
	return "/api/media/" + escaped
}

func (c *ZealotClient) uploadMediaFile(localPath, destinationDir string) error {
	file, err := os.Open(localPath)
	if err != nil {
		return err
	}
	defer file.Close()

	var body bytes.Buffer
	writer := multipart.NewWriter(&body)

	part, err := writer.CreateFormFile("file", filepath.Base(localPath))
	if err != nil {
		return err
	}
	if _, err := io.Copy(part, file); err != nil {
		return err
	}
	if err := writer.Close(); err != nil {
		return err
	}

	req, err := http.NewRequest(http.MethodPost, c.apiURL(c.mediaPath(destinationDir)), &body)
	if err != nil {
		return err
	}
	req.Header.Set("Content-Type", writer.FormDataContentType())

	return c.do(req, nil)
}

func (c *ZealotClient) GetToday() ([]Item, error) {
	today := time.Now().Format(time.DateOnly)
	return c.FilterItems([]AttributeFilter{
		{Key: "Date", Op: "eq", Value: today},
	})
}

func (c *ZealotClient) GetActiveTickets() ([]Item, error) {
	return c.FilterItems([]AttributeFilter{
		{Key: "Status", Op: "eq", Value: "Working"},
	})
}

func (c *ZealotClient) GetAccountDetails() (*AccountDetails, error) {
	var details AccountDetails
	if err := c.request(http.MethodGet, "/api/account/details", nil, &details); err != nil {
		return nil, err
	}
	return &details, nil
}

func (c *ZealotClient) GetAPIKeyStatus() (*APIKeyStatus, error) {
	var status APIKeyStatus
	if err := c.request(http.MethodGet, "/api/account/api-key", nil, &status); err != nil {
		return nil, err
	}
	return &status, nil
}

func (c *ZealotClient) UpdateAccountSettings(settings json.RawMessage) error {
	return c.request(http.MethodPatch, "/api/account/settings", settings, nil)
}

func (c *ZealotClient) GetItemByID(id int) (*Item, error) {
	var item Item
	if err := c.request(http.MethodGet, fmt.Sprintf("/api/item/id/%d", id), nil, &item); err != nil {
		return nil, err
	}
	return &item, nil
}

func (c *ZealotClient) GetItemByTitle(title string) (*Item, error) {
	var item Item
	if err := c.request(http.MethodGet, "/api/item/title/"+url.PathEscape(title), nil, &item); err != nil {
		return nil, err
	}
	return &item, nil
}

func (c *ZealotClient) SearchItems(query string) ([]Item, error) {
	var items []Item
	err := c.request(http.MethodGet, "/api/item/search?term="+url.QueryEscape(query), nil, &items)
	return items, err
}

func (c *ZealotClient) GetChildren(itemID int) ([]Item, error) {
	var items []Item
	err := c.request(http.MethodGet, fmt.Sprintf("/api/item/children/%d", itemID), nil, &items)
	return items, err
}

func (c *ZealotClient) GetRelatedItems(itemID int) ([]Item, error) {
	var items []Item
	err := c.request(http.MethodGet, fmt.Sprintf("/api/item/related/%d", itemID), nil, &items)
	return items, err
}

func (c *ZealotClient) CreateItem(input CreateItemInput) (*Item, error) {
	payload := Item{
		Title:      input.Title,
		Attributes: input.Attributes,
	}
	if payload.Attributes == nil {
		payload.Attributes = map[string]any{}
	}

	var created Item
	if err := c.request(http.MethodPost, "/api/item/", payload, &created); err != nil {
		return nil, err
	}

	if input.Content != "" {
		if err := c.UpdateItemFields(created.ItemID, map[string]any{"content": input.Content}); err != nil {
			return nil, err
		}
	}

	for _, typeName := range input.TypeNames {
		if err := c.AssignItemType(created.ItemID, typeName); err != nil {
			return nil, err
		}
	}

	if input.Content != "" || len(input.TypeNames) > 0 {
		return c.GetItemByID(created.ItemID)
	}
	return &created, nil
}

func (c *ZealotClient) UpdateItemFields(itemID int, updates map[string]any) error {
	if len(updates) == 0 {
		return fmt.Errorf("at least one item field update is required")
	}
	return c.request(http.MethodPatch, fmt.Sprintf("/api/item/%d", itemID), updates, nil)
}

func (c *ZealotClient) DeleteItem(itemID int) error {
	return c.request(http.MethodDelete, fmt.Sprintf("/api/item/%d", itemID), nil, nil)
}

func (c *ZealotClient) SetItemAttributes(itemID int, attributes map[string]any) error {
	if len(attributes) == 0 {
		return fmt.Errorf("at least one attribute is required")
	}
	return c.request(http.MethodPatch, fmt.Sprintf("/api/item/%d/attr", itemID), attributes, nil)
}

func (c *ZealotClient) RenameItemAttribute(itemID int, oldKey, newKey string) error {
	payload := map[string]string{
		"old_key": oldKey,
		"new_key": newKey,
	}
	return c.request(http.MethodPatch, fmt.Sprintf("/api/item/%d/attr/rename", itemID), payload, nil)
}

func (c *ZealotClient) DeleteItemAttribute(itemID int, key string) error {
	return c.request(http.MethodDelete, fmt.Sprintf("/api/item/%d/attr/%s", itemID, url.PathEscape(key)), nil, nil)
}

func (c *ZealotClient) FilterItems(filters []AttributeFilter) ([]Item, error) {
	payload := struct {
		Filters []AttributeFilter `json:"filters"`
	}{Filters: filters}

	var items []Item
	err := c.request(http.MethodPost, "/api/item/filter", payload, &items)
	return items, err
}

func (c *ZealotClient) GetItemsByType(typeName string) ([]Item, error) {
	var items []Item
	err := c.request(http.MethodGet, "/api/item/?type="+url.QueryEscape(typeName), nil, &items)
	return items, err
}

func (c *ZealotClient) AssignItemType(itemID int, typeName string) error {
	return c.request(http.MethodPost, fmt.Sprintf("/api/item/%d/assign_type/%s", itemID, url.PathEscape(typeName)), nil, nil)
}

func (c *ZealotClient) UnassignItemType(itemID int, typeName string) error {
	return c.request(http.MethodDelete, fmt.Sprintf("/api/item/%d/assign_type/%s", itemID, url.PathEscape(typeName)), nil, nil)
}

func (c *ZealotClient) ListItemTypes() ([]ItemType, error) {
	var types []ItemType
	err := c.request(http.MethodGet, "/api/item/type/", nil, &types)
	return types, err
}

func (c *ZealotClient) CreateItemType(input CreateItemTypeInput) error {
	payload := ItemType{
		Name:        input.Name,
		Description: input.Description,
	}
	if err := c.request(http.MethodPost, "/api/item/type/", payload, nil); err != nil {
		return err
	}
	if len(input.RequiredAttributeKeys) == 0 {
		return nil
	}
	return c.AddAttributeKindsToItemType(input.Name, input.RequiredAttributeKeys)
}

func (c *ZealotClient) UpdateItemType(typeID int, updates map[string]any) error {
	if len(updates) == 0 {
		return fmt.Errorf("at least one item type field update is required")
	}
	return c.request(http.MethodPatch, fmt.Sprintf("/api/item/type/%d", typeID), updates, nil)
}

func (c *ZealotClient) DeleteItemType(typeID int) error {
	return c.request(http.MethodDelete, fmt.Sprintf("/api/item/type/%d", typeID), nil, nil)
}

func (c *ZealotClient) AddAttributeKindsToItemType(typeName string, attributeKinds []string) error {
	payload := map[string][]string{
		"attribute_kinds": attributeKinds,
	}
	return c.request(http.MethodPost, "/api/item/type/assign/"+url.PathEscape(typeName), payload, nil)
}

func (c *ZealotClient) RemoveAttributeKindsFromItemType(typeName string, attributeKinds []string) error {
	payload := map[string][]string{
		"attribute_kinds": attributeKinds,
	}
	return c.request(http.MethodDelete, "/api/item/type/assign/"+url.PathEscape(typeName), payload, nil)
}

func (c *ZealotClient) ListAttributeKinds() ([]AttributeKind, error) {
	var kinds []AttributeKind
	err := c.request(http.MethodGet, "/api/item/kind/", nil, &kinds)
	return kinds, err
}

func (c *ZealotClient) CreateAttributeKind(kind AttributeKind) error {
	return c.request(http.MethodPost, "/api/item/kind/", kind, nil)
}

func (c *ZealotClient) UpdateAttributeKind(kindID int, updates map[string]any) error {
	if len(updates) == 0 {
		return fmt.Errorf("at least one attribute kind field update is required")
	}
	return c.request(http.MethodPatch, fmt.Sprintf("/api/item/kind/%d", kindID), updates, nil)
}

func (c *ZealotClient) UpdateAttributeKindConfig(kindID int, config json.RawMessage) error {
	payload := struct {
		Config json.RawMessage `json:"config"`
	}{Config: config}
	return c.request(http.MethodPatch, fmt.Sprintf("/api/item/kind/%d/config", kindID), payload, nil)
}

func (c *ZealotClient) DeleteAttributeKind(kindID int) error {
	return c.request(http.MethodDelete, fmt.Sprintf("/api/item/kind/%d", kindID), nil, nil)
}

func (c *ZealotClient) GetPlannerDay(date string) ([]Item, error) {
	var items []Item
	err := c.request(http.MethodGet, "/api/planner/day/"+url.PathEscape(date), nil, &items)
	return items, err
}

func (c *ZealotClient) GetPlannerWeek(week string) ([]Item, error) {
	var items []Item
	err := c.request(http.MethodGet, "/api/planner/week/"+url.PathEscape(week), nil, &items)
	return items, err
}

func (c *ZealotClient) GetPlannerMonth(month, year int) ([]Item, error) {
	var items []Item
	err := c.request(http.MethodGet, fmt.Sprintf("/api/planner/month/%d/year/%d", month, year), nil, &items)
	return items, err
}

func (c *ZealotClient) GetPlannerYear(year int) ([]Item, error) {
	var items []Item
	err := c.request(http.MethodGet, fmt.Sprintf("/api/planner/year/%d", year), nil, &items)
	return items, err
}

func (c *ZealotClient) GetRepeatsForDay(date string) ([]RepeatStatusDate, error) {
	var repeats []RepeatStatusDate
	err := c.request(http.MethodGet, "/api/repeat/day/"+url.PathEscape(date), nil, &repeats)
	return repeats, err
}

func (c *ZealotClient) SetRepeatStatus(itemID int, date, status, comment string) error {
	payload := map[string]string{
		"status":  status,
		"comment": comment,
	}
	return c.request(http.MethodPatch, fmt.Sprintf("/api/repeat/%d/day/%s", itemID, url.PathEscape(date)), payload, nil)
}

func (c *ZealotClient) GetCommentsForDay(date string) ([]CommentEntry, error) {
	var entries []CommentEntry
	err := c.request(http.MethodGet, "/api/comments/day/"+url.PathEscape(date), nil, &entries)
	return entries, err
}

func (c *ZealotClient) GetCommentsForItem(itemID int) ([]CommentEntry, error) {
	var entries []CommentEntry
	err := c.request(http.MethodGet, fmt.Sprintf("/api/comments/item/%d", itemID), nil, &entries)
	return entries, err
}

func (c *ZealotClient) AddComment(itemID int, timestamp, content string) error {
	payload := map[string]any{
		"item_id": itemID,
		"content": content,
	}
	if timestamp != "" {
		payload["timestamp"] = timestamp
	}
	return c.request(http.MethodPost, "/api/comments/", payload, nil)
}

func (c *ZealotClient) UpdateComment(commentID int64, content, timestamp string) error {
	payload := map[string]any{
		"content": content,
	}
	if timestamp != "" {
		payload["time"] = timestamp
	}
	return c.request(http.MethodPatch, fmt.Sprintf("/api/comments/%d", commentID), payload, nil)
}

func (c *ZealotClient) DeleteComment(commentID int64) error {
	return c.request(http.MethodDelete, fmt.Sprintf("/api/comments/%d", commentID), nil, nil)
}

func (c *ZealotClient) GetTrackerDay(date string) ([]TrackerEntry, error) {
	var entries []TrackerEntry
	err := c.request(http.MethodGet, "/api/tracker/day/"+url.PathEscape(date), nil, &entries)
	return entries, err
}

func (c *ZealotClient) AddTrackerEntry(itemID int, timestamp string, level int, comment string) error {
	payload := map[string]any{
		"item_id": itemID,
		"comment": comment,
	}
	if timestamp != "" {
		payload["timestamp"] = timestamp
	}
	if level != 0 {
		payload["level"] = level
	}
	return c.request(http.MethodPost, "/api/tracker/", payload, nil)
}

func (c *ZealotClient) DeleteTrackerEntry(trackerID int64) error {
	return c.request(http.MethodDelete, fmt.Sprintf("/api/tracker/%d", trackerID), nil, nil)
}

func (c *ZealotClient) GetScoreHistory() ([]ScoreEntry, error) {
	var entries []ScoreEntry
	err := c.request(http.MethodGet, "/api/analysis/last", nil, &entries)
	return entries, err
}

func (c *ZealotClient) ListMedia(path string) (*MediaDirectory, error) {
	var listing MediaDirectory
	if err := c.request(http.MethodGet, c.mediaPath(path), nil, &listing); err != nil {
		return nil, err
	}
	return &listing, nil
}

func (c *ZealotClient) CreateMediaFolder(folder string) error {
	payload := map[string]string{"folder": folder}
	return c.request(http.MethodPost, "/api/media/mkdir", payload, nil)
}

func (c *ZealotClient) UploadMediaFile(localPath, destinationDir string) error {
	return c.uploadMediaFile(localPath, destinationDir)
}

func (c *ZealotClient) RenameMediaEntry(oldLocation, newName string) error {
	payload := map[string]string{
		"old_location": oldLocation,
		"new_name":     newName,
	}
	return c.request(http.MethodPatch, "/api/media/rename", payload, nil)
}

func (c *ZealotClient) DeleteMediaEntry(path string) error {
	return c.request(http.MethodDelete, c.mediaPath(path), nil, nil)
}
