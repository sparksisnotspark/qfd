package response

import "github.com/gin-gonic/gin"

type Body struct {
	Code int         `json:"code"`
	Msg  string      `json:"msg"`
	Data interface{} `json:"data"`
}

func Success(c *gin.Context, data interface{}) {
	c.JSON(200, Body{Code: 0, Msg: "success", Data: data})
}

func SuccessWithMsg(c *gin.Context, msg string, data interface{}) {
	c.JSON(200, Body{Code: 0, Msg: msg, Data: data})
}

func Failure(c *gin.Context, code int, msg string, data interface{}) {
	c.JSON(200, Body{Code: code, Msg: msg, Data: data})
}
