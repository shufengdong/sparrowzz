package com.jinyun.web.controller;

import com.jinyun.web.service.CapService;
import io.swagger.annotations.Api;
import io.swagger.annotations.ApiImplicitParam;
import io.swagger.annotations.ApiImplicitParams;
import io.swagger.annotations.ApiOperation;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.*;

import java.util.List;
import java.util.Map;

@Api(value = "desc of class", tags="LineBase",description = "线路台区")
@RestController
@RequestMapping("lineBase")
public class LineBaseController {
    @Autowired
    CapService capService;

    @ApiOperation(value = "基础信息", notes = "")
    @ApiImplicitParams({
            @ApiImplicitParam(name = "mRID", value = "开关ID", dataType = "string", paramType = "query",required = true),
    })
    @RequestMapping(value = "/baseInfo",method = RequestMethod.GET)
    public Object baseInfo(@RequestParam("mRID") String mRID) {
        Map<String,Object> result = capService.baseInfo(mRID);
        return result;
    }

    @ApiOperation(value = "夏季分析", notes = "")
    @ApiImplicitParams({
            @ApiImplicitParam(name = "mRID", value = "开关ID", dataType = "string", paramType = "query",required = true),
    })
    @RequestMapping(value = "/lineSummer",method = RequestMethod.GET)
    public Object lineSummer(@RequestParam("mRID") String mRID) {
        Map<String,Object> result = capService.lineSummer(mRID);
        return result;
    }

    @ApiOperation(value = "冬季分析", notes = "")
    @ApiImplicitParams({
            @ApiImplicitParam(name = "mRID", value = "开关ID", dataType = "string", paramType = "query",required = true),
    })
    @RequestMapping(value = "/lineWinter",method = RequestMethod.GET)
    public Object lineWinter(@RequestParam("mRID") String mRID) {
        Map<String,Object> result = capService.lineWinter(mRID);
        return result;
    }

    @ApiOperation(value = "峰谷分析", notes = "")
    @ApiImplicitParams({
            @ApiImplicitParam(name = "mRID", value = "开关ID", dataType = "string", paramType = "query",required = true),
    })
    @RequestMapping(value = "/linePs",method = RequestMethod.GET)
    public Object linePs(@RequestParam("mRID") String mRID) {
        Map<String,Object> result = capService.linePs(mRID);
        return result;
    }
}
