package app

type Deps struct {
	Items item.Repository
	Types item.TypeRepository
	Kinds item.AttributeKindRepository
	Comments item.CommentRepository
	Tx item.UnitOfWork
	Pub item.Publisher
}

type Service struct {
	items item.Repository
	types item.TypeRepository
	kinds item.AttributeKindRepository
	comments item.CommentRepository
	tx item.UnitOfWork
	pub item.Publisher
}

func NewService(d Deps) *Service {
	return &Service{
		items: Items,
		types: Types,
		kinds: Kinds,
		comments: Comments,
		tx: Tx,
		pub: Pub,
	}
}