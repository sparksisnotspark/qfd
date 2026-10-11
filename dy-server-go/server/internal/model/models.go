package model

import "time"

type DyCaiji struct {
	ID        uint         `gorm:"column:id" json:"id"`
	CJName    string       `gorm:"column:cj_name" json:"cj_name"`
	CJCount   int          `gorm:"column:cj_count" json:"cj_count"`
	UpCount   int          `gorm:"column:up_count" json:"up_count"`
	CJRemark  *string      `gorm:"column:cj_remark" json:"cj_remark"`
	CJStatus  int          `gorm:"column:cj_status" json:"cj_status"`
	CreatedAt *time.Time   `gorm:"column:created_at" json:"created_at"`
	UpdatedAt *time.Time   `gorm:"column:updated_at" json:"updated_at"`
	Cookies   []DyCookies  `gorm:"many2many:dy_cj_ck;joinForeignKey:CJID;joinReferences:CKID" json:"cookies,omitempty"`
}

func (DyCaiji) TableName() string { return "dy_caiji" }

type DyCJCK struct {
	ID        uint       `gorm:"column:id"`
	CJID      uint       `gorm:"column:cj_id"`
	CKID      uint       `gorm:"column:ck_id"`
	CreatedAt *time.Time `gorm:"column:created_at"`
	UpdatedAt *time.Time `gorm:"column:updated_at"`
}

func (DyCJCK) TableName() string { return "dy_cj_ck" }

type DyCookies struct {
	ID        uint       `gorm:"column:id" json:"id"`
	Cookie    string     `gorm:"column:cookie" json:"cookie"`
	DYName    string     `gorm:"column:dy_name" json:"dy_name"`
	InUsed    int        `gorm:"column:in_used" json:"in_used"`
	CanUse    int        `gorm:"column:can_use" json:"can_use"`
	CKRemark  *string    `gorm:"column:ck_remark" json:"ck_remark"`
	CreatedAt *time.Time `gorm:"column:created_at" json:"created_at"`
	UpdatedAt *time.Time `gorm:"column:updated_at" json:"updated_at"`
	Caiji     []DyCaiji  `gorm:"many2many:dy_cj_ck;joinForeignKey:CKID;joinReferences:CJID" json:"caiji,omitempty"`
	CJName    *string    `gorm:"-" json:"cj_name,omitempty"`
	UsedInfo  *string    `gorm:"-" json:"used_info,omitempty"`
	TodyUsed  *string    `gorm:"-" json:"tody_used,omitempty"`
}

func (DyCookies) TableName() string { return "dy_cookies" }

type DyDailyLog struct {
	ID        uint   `gorm:"column:id" json:"id"`
	Nickname  string `gorm:"column:nickname" json:"nickname"`
	KeyID     *int   `gorm:"column:key_id" json:"key_id"`
	DYNo      string `gorm:"column:dy_no" json:"dy_no"`
	FDCS      int    `gorm:"column:fd_cs" json:"fd_cs"`
	FDZCS     int    `gorm:"column:fd_z_cs" json:"fd_z_cs"`
	FDXHZS    int    `gorm:"column:fd_xh_zs" json:"fd_xh_zs"`
	FDHDZS    int    `gorm:"column:fd_hd_zs" json:"fd_hd_zs"`
	HBCS      int    `gorm:"column:hb_cs" json:"hb_cs"`
	HBZCS     int    `gorm:"column:hb_z_cs" json:"hb_z_cs"`
	HBXHZS    int    `gorm:"column:hb_xh_zs" json:"hb_xh_zs"`
	HBHDZS    int    `gorm:"column:hb_hd_zs" json:"hb_hd_zs"`
	SSZS      int    `gorm:"column:ss_zs" json:"ss_zs"`
	CreatedAt string `gorm:"column:created_at" json:"created_at"`
}

func (DyDailyLog) TableName() string { return "dy_daily_log" }

type KeysManage struct {
	ID        uint       `gorm:"column:id" json:"id"`
	KeyCode   string     `gorm:"column:key_code" json:"key_code"`
	UserID    *uint      `gorm:"column:user_id" json:"user_id"`
	ExpiredAt *time.Time `gorm:"column:expired_at" json:"expired_at"`
	CreatedAt *time.Time `gorm:"column:created_at" json:"created_at"`
	UpdatedAt *time.Time `gorm:"column:updated_at" json:"updated_at"`
	IsExpired int        `gorm:"column:is_expired" json:"is_expired"`
}

func (KeysManage) TableName() string { return "keys_manage" }

type KeyStore struct {
	ID                 uint        `gorm:"column:id" json:"id"`
	KID                *uint       `gorm:"column:kid" json:"kid"`
	KeyCode            string      `gorm:"column:key_code" json:"key_code"`
	ExpirationDate     int         `gorm:"column:expiration_date" json:"expiration_date"`
	ExpirationDateUnit string      `gorm:"column:expiration_date_unit" json:"expiration_date_unit"`
	IsUsed             int         `gorm:"column:is_used" json:"is_used"`
	Remark             string      `gorm:"column:remark" json:"remark"`
	CreatedAt          *time.Time  `gorm:"column:created_at" json:"created_at"`
	UpdatedAt          *time.Time  `gorm:"-" json:"updated_at"`
	KeysManage         *KeysManage `gorm:"foreignKey:KID" json:"keys_manage,omitempty"`
}

func (KeyStore) TableName() string { return "key_store" }

type RoomURL struct {
	ID        uint   `gorm:"column:id" json:"id"`
	RoomURL   string `gorm:"column:room_url" json:"room_url"`
	KeyID     *uint  `gorm:"column:key_id" json:"key_id"`
	RoomID    string `gorm:"column:room_id" json:"room_id"`
	WebRID    string `gorm:"column:web_rid" json:"web_rid"`
	Title     string `gorm:"column:title" json:"title"`
	TP        string `gorm:"column:tp" json:"tp"`
	SendTime  int64  `gorm:"column:send_time" json:"send_time"`
	UsedTimes int    `gorm:"column:used_times" json:"used_times"`
	Delay     int    `gorm:"column:delay" json:"delay"`
	ExpiredAt int64  `gorm:"column:expired_at" json:"expired_at"`
}

func (RoomURL) TableName() string { return "room_url" }

type User struct {
	ID           uint       `gorm:"column:id" json:"id"`
	Username     string     `gorm:"column:username" json:"username"`
	Password     string     `gorm:"column:password" json:"password"`
	PasswordSalt *string    `gorm:"column:password_salt" json:"password_salt"`
	MachineID    *string    `gorm:"column:machine_id" json:"machine_id"`
	ExpiredAt    *time.Time `gorm:"column:expired_at" json:"expired_at"`
	CreatedAt    *time.Time `gorm:"column:created_at" json:"created_at"`
	UpdatedAt    *time.Time `gorm:"column:updated_at" json:"updated_at"`
}

func (User) TableName() string { return "users" }

type DictOption struct {
	ID        uint       `gorm:"column:id" json:"id"`
	Name      string     `gorm:"column:name" json:"name"`
	Value     string     `gorm:"column:value" json:"value"`
	CreatedAt *time.Time `gorm:"column:created_at" json:"created_at"`
	UpdatedAt *time.Time `gorm:"column:updated_at" json:"updated_at"`
}

func (DictOption) TableName() string { return "wa_options" }
