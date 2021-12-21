package com.jinyun.web.controller;

import com.jinyun.web.annotation.UserLoginToken;
import com.jinyun.web.service.CapService;
import io.swagger.annotations.Api;
import io.swagger.annotations.ApiImplicitParam;
import io.swagger.annotations.ApiImplicitParams;
import io.swagger.annotations.ApiOperation;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestMethod;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

@Api(value = "desc of class", tags="LoadPos",description = "负荷接入分析")
@RestController
@RequestMapping("loadPos")
public class LoadPosController {
    @Autowired
    CapService capService;

    @ApiOperation(value = "分析结果列表", notes = "")
    @ApiImplicitParams({
            @ApiImplicitParam(name = "page", value = "页码", dataType = "int", paramType = "query",example = "0",defaultValue = "0",required = true),
            @ApiImplicitParam(name = "rows", value = "行数", dataType = "int", paramType = "query",example = "0",defaultValue = "0",required = true),
    })
    @RequestMapping(value = "/loadPosList",method = RequestMethod.GET)
    @UserLoginToken
    public Object loadPosSeason(@RequestParam("page") int page,@RequestParam("rows") int rows) {
        int total = capService.loadPosListCout();
        List loadPosList = new ArrayList();
        if(total>0){
            loadPosList = capService.loadPosList(page,rows);
        }
        Map<String,Object> result = new HashMap<>();
        result.put("total",total);
        result.put("rows",loadPosList);
        return result;
    }

    @ApiOperation(value = "线路名称列表", notes = "")
    @RequestMapping(value = "/lineNameList",method = RequestMethod.GET)
    @UserLoginToken
    public Object lineNameList() {
        List result = capService.lineNameList();
        return result;
    }

    @ApiOperation(value = "负荷接入分析", notes = "")
    @ApiImplicitParams({
            @ApiImplicitParam(name = "lineName", value = "线路名称", dataType = "string", paramType = "query",required = true),
            @ApiImplicitParam(name = "cap", value = "负荷容量", dataType = "int", paramType = "query",example = "0",required = true),
            @ApiImplicitParam(name = "type", value = "负荷特征(1为峰用电，2为谷用电，3为峰谷用电)", dataType = "int", paramType = "query",example = "0",required = true),
    })
    @RequestMapping(value = "/loadPosAnalysis",method = RequestMethod.GET)
    @UserLoginToken
    public Object loadPosAnalysis(@RequestParam("lineName") String lineName,@RequestParam("cap") int cap,@RequestParam("type") int type) {
        Object result = capService.loadPosAnalysis(lineName, cap, type);
        if(result != null)
            return "分析完成";
        else return "分析失败";
    }

    @ApiOperation(value = "高压负荷接入分析结果", notes = "")
    @ApiImplicitParams({
            @ApiImplicitParam(name = "loadId", value = "负荷ID", dataType = "string", paramType = "query",required = true),
    })
    @RequestMapping(value = "/loadPosSw",method = RequestMethod.GET)
    @UserLoginToken
    public Object loadPosSw(@RequestParam("loadId") String loadId) {
        Object result = capService.loadPosSw(loadId);
        return result;
    }

    @ApiOperation(value = "低压负荷接入分析结果", notes = "")
    @ApiImplicitParams({
            @ApiImplicitParam(name = "loadId", value = "负荷ID", dataType = "string", paramType = "query",required = true),
    })
    @RequestMapping(value = "/loadPosTf",method = RequestMethod.GET)
    @UserLoginToken
    public Object loadPosTf(@RequestParam("loadId") String loadId) {
        Object result = capService.loadPosTf(loadId);
        return result;
    }
}
