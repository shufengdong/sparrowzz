package com.jinyun.web.controller;

import com.jinyun.web.annotation.UserLoginToken;
import com.jinyun.web.service.CapService;
import io.swagger.annotations.Api;
import io.swagger.annotations.ApiImplicitParam;
import io.swagger.annotations.ApiImplicitParams;
import io.swagger.annotations.ApiOperation;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.*;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

@Api(value = "desc of class", tags="MainPage",description = "首页")
@RestController
@RequestMapping("mainPage")
public class MainPageController {
    @Autowired
    CapService capService;

    @ApiOperation(value = "电网统计字段", notes = "")
    @RequestMapping(value = "/gridStatistics",method = RequestMethod.GET)
    @UserLoginToken
    public Object gridStatistics() {
        Map<String,Object> result = capService.gridStatistics();
        return result;
    }

    @ApiOperation(value = "低裕度预警统计", notes = "")
    @RequestMapping(value = "/lowMarginWarnStatistics",method = RequestMethod.GET)
    @UserLoginToken
    public Object lowMarginWarnStatistics() {
        Map<String,Object> result = capService.lowMarginWarnStatistics();
        return result;
    }

    @ApiOperation(value = "重载线路列表", notes = "")
    @ApiImplicitParams({
            @ApiImplicitParam(name = "page", value = "页码", dataType = "int", paramType = "query",example = "0",defaultValue = "0",required = true),
            @ApiImplicitParam(name = "rows", value = "行数", dataType = "int", paramType = "query",example = "0",defaultValue = "0",required = true),
    })
    @RequestMapping(value = "/hardLineList",method = RequestMethod.GET)
    @UserLoginToken
    public Object hardLineList(@RequestParam("page") int page,@RequestParam("rows") int rows) {
        int total = capService.hardLineListCount();
        List hardLineList = new ArrayList();
        if(total>0){
            hardLineList = capService.hardLineList(page,rows);
        }
        Map<String,Object> result = new HashMap<>();
        result.put("total",total);
        result.put("rows",hardLineList);
        return result;
    }

    @ApiOperation(value = "重载配变列表", notes = "")
    @ApiImplicitParams({
            @ApiImplicitParam(name = "page", value = "页码", dataType = "int", paramType = "query",example = "0",defaultValue = "0",required = true),
            @ApiImplicitParam(name = "rows", value = "行数", dataType = "int", paramType = "query",example = "0",defaultValue = "0",required = true),
    })
    @RequestMapping(value = "/hardTransformerList",method = RequestMethod.GET)
    @UserLoginToken
    public Object hardTransformerList(@RequestParam("page") int page,@RequestParam("rows") int rows) {
        int total = capService.hardTransformerListCount();
        List hardTransformerList = new ArrayList();
        if(total>0){
            hardTransformerList = capService.hardTransformerList(page,rows);
        }
        Map<String,Object> result = new HashMap<>();
        result.put("total",total);
        result.put("rows",hardTransformerList);
        return result;
    }

    @ApiOperation(value = "超载线路列表", notes = "")
    @ApiImplicitParams({
            @ApiImplicitParam(name = "page", value = "页码", dataType = "int", paramType = "query",example = "0",defaultValue = "0",required = true),
            @ApiImplicitParam(name = "rows", value = "行数", dataType = "int", paramType = "query",example = "0",defaultValue = "0",required = true),
    })
    @RequestMapping(value = "/overLineList",method = RequestMethod.GET)
    @UserLoginToken
    public Object overLineList(@RequestParam("page") int page,@RequestParam("rows") int rows) {
        int total = capService.overLineListCount();
        List overLineList = new ArrayList();
        if(total>0){
            overLineList = capService.overLineList(page,rows);
        }
        Map<String,Object> result = new HashMap<>();
        result.put("total",total);
        result.put("rows",overLineList);
        return result;
    }

    @ApiOperation(value = "超载配变列表", notes = "")
    @ApiImplicitParams({
            @ApiImplicitParam(name = "page", value = "页码", dataType = "int", paramType = "query",example = "0",defaultValue = "0",required = true),
            @ApiImplicitParam(name = "rows", value = "行数", dataType = "int", paramType = "query",example = "0",defaultValue = "0",required = true),
    })
    @RequestMapping(value = "/overTransformerList",method = RequestMethod.GET)
    @UserLoginToken
    public Object overTransformerList(@RequestParam("page") int page,@RequestParam("rows") int rows) {
        int total = capService.overTransformerListCount();
        List overTransformerList = new ArrayList();
        if(total>0){
            overTransformerList = capService.overTransformerList(page,rows);
        }
        Map<String,Object> result = new HashMap<>();
        result.put("total",total);
        result.put("rows",overTransformerList);
        return result;
    }


    @ApiOperation(value = "配变三相不平衡度", notes = "")
    @RequestMapping(value = "/transformerUnbalance",method = RequestMethod.GET)
    @UserLoginToken
    public Object transformerUnbalance() {
        Map<String,Object> result = capService.transformerUnbalance();
        return result;
    }

    @ApiOperation(value = "配变三相不平衡度列表", notes = "")
    @ApiImplicitParams({
            @ApiImplicitParam(name = "page", value = "页码", dataType = "int", paramType = "query",example = "0",defaultValue = "0",required = true),
            @ApiImplicitParam(name = "rows", value = "行数", dataType = "int", paramType = "query",example = "0",defaultValue = "0",required = true),
    })
    @RequestMapping(value = "/transformerUnbalanceList",method = RequestMethod.GET)
    @UserLoginToken
    public Object transformerUnbalanceList(@RequestParam("page") int page,@RequestParam("rows") int rows) {
        Map<String,Object> result = capService.transformerUnbalanceList(page,rows);
        return result;
    }

    @ApiOperation(value = "线路电流合格率", notes = "")
    @RequestMapping(value = "/linePassRate",method = RequestMethod.GET)
    @UserLoginToken
    public Object linePassRate() {
        List result = capService.linePassRate();
        return result;
    }

    @ApiOperation(value = "配变负荷越限统计", notes = "")
    @RequestMapping(value = "/transformerOverLoadRate",method = RequestMethod.GET)
    @UserLoginToken
    public Object transformerOverLoadRate() {
        Object result = capService.transformerOverLoadRate();
        return result;
    }

    @ApiOperation(value = "线路可开放容量大小", notes = "")
    @RequestMapping(value = "/maxMinAvailCap",method = RequestMethod.GET)
    @UserLoginToken
    public Object maxMinAvailCap() {
        Map<String, Object> result = capService.maxMinAvailCap();
        return result;
    }
}
