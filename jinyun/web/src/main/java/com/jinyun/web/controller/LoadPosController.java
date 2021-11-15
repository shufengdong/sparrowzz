package com.jinyun.web.controller;

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

import java.util.List;
import java.util.Map;

@Api(value = "desc of class", tags="LoadPos",description = "负荷接入分析")
@RestController
@RequestMapping("loadPos")
public class LoadPosController {
    @Autowired
    CapService capService;

    @ApiOperation(value = "接入分析", notes = "")
    @RequestMapping(value = "/loadPosList",method = RequestMethod.GET)
    public Object loadPosSeason() {
        List result = capService.loadPosList();
        return result;
    }

    @ApiOperation(value = "高压负荷接入分析结果", notes = "")
    @ApiImplicitParams({
            @ApiImplicitParam(name = "mRID", value = "负荷ID", dataType = "string", paramType = "query",required = true),
    })
    @RequestMapping(value = "/loadPosSw",method = RequestMethod.GET)
    public Object loadPosSw(@RequestParam("mRID") String mRID) {
        Object result = capService.loadPosSw(mRID);
        return result;
    }

    @ApiOperation(value = "低压负荷接入分析结果", notes = "")
    @ApiImplicitParams({
            @ApiImplicitParam(name = "mRID", value = "负荷ID", dataType = "string", paramType = "query",required = true),
    })
    @RequestMapping(value = "/loadPosTf",method = RequestMethod.GET)
    public Object loadPosTf(@RequestParam("mRID") String mRID) {
        Object result = capService.loadPosTf(mRID);
        return result;
    }
}
